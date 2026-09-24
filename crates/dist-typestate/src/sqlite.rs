use rusqlite::Connection;
use std::sync::Mutex;

use crate::backend::StateBackend;
use crate::error::TransitionError;

/// SQLiteによるStateBackend実装
///
/// CASの部分:
///   UPDATE ... WHERE state = ? AND revision = ?
///   affected rows == 0 → StaleCapability
pub struct SqliteBackend {
    conn: Mutex<Connection>,
}

impl SqliteBackend {
    pub fn new(path: &str) -> Result<Self, TransitionError> {
        let conn = Connection::open(path)
            .map_err(|e| TransitionError::BackendError(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS resources (
                id       TEXT PRIMARY KEY,
                state    TEXT NOT NULL,
                revision INTEGER NOT NULL
            );",
        )
        .map_err(|e| TransitionError::BackendError(e.to_string()))?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    /// テスト用: インメモリDBで作成
    pub fn in_memory() -> Result<Self, TransitionError> {
        Self::new(":memory:")
    }
}

impl StateBackend for SqliteBackend {
    fn load(&self, resource_id: &str) -> Result<(String, u64), TransitionError> {
        let conn = self.conn.lock().unwrap();
        conn
            .query_row(
                "SELECT state, revision FROM resources WHERE id = ?1",
                [resource_id],
                |row| {
                    let state: String = row.get(0)?;
                    let revision: u64 = row.get(1)?;
                    Ok((state, revision))
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => TransitionError::NotFound {
                    id: resource_id.to_string(),
                },
                other => TransitionError::BackendError(other.to_string()),
            })
    }

    fn compare_and_transition(
        &self,
        resource_id: &str,
        expected_state: &str,
        expected_revision: u64,
        next_state: &str,
    ) -> Result<u64, TransitionError> {
        let new_revision = expected_revision + 1;

        // CASの部分
        let affected = {
            let conn = self.conn.lock().unwrap();
            conn
                .execute(
                    "UPDATE resources
                     SET state = ?1, revision = ?2
                     WHERE id = ?3 AND state = ?4 AND revision = ?5",
                    (
                        next_state,
                        new_revision,
                        resource_id,
                        expected_state,
                        expected_revision,
                    ),
                )
                .map_err(|e| TransitionError::BackendError(e.to_string()))?
        };

        if affected == 0 {
            // CAS失敗: 現在値を取得してエラーに含める
            let actual = self.load(resource_id).ok();
            return Err(TransitionError::StaleCapability {
                expected_revision,
                actual_revision: actual.map(|(_, r)| r),
            });
        }

        Ok(new_revision)
    }

    fn create(
        &self,
        resource_id: &str,
        initial_state: &str,
    ) -> Result<u64, TransitionError> {
        let initial_revision = 1u64;

        let conn = self.conn.lock().unwrap();
        conn
            .execute(
                "INSERT INTO resources (id, state, revision) VALUES (?1, ?2, ?3)",
                (resource_id, initial_state, initial_revision),
            )
            .map_err(|e| {
                // UNIQUE制約違反: AlreadyExists
                if e.to_string().contains("UNIQUE") {
                    TransitionError::AlreadyExists {
                        id: resource_id.to_string(),
                    }
                } else {
                    TransitionError::BackendError(e.to_string())
                }
            })?;

        Ok(initial_revision)
    }
}

