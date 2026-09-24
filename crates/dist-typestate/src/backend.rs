use crate::error::TransitionError;

/// 状態の永続化と CAS 遷移を担うバックエンド
///
/// 最低限atomicなcompare-and-swap相当の操作を提供できる
/// ストレージであれば実装可能。
/// SQLite対応
/// Redisに対応予定
pub trait StateBackend {
    /// リソースの現在のstate, revisionを取得する
    fn load(&self, resource_id: &str) -> Result<(String, u64), TransitionError>;

    /// CAS遷移
    ///
    /// expected_stateかつexpected_revisionに一致する場合のみ
    /// next_stateへ遷移し、新しいrevisionを返す。
    ///
    /// 不一致の場合はTransitionError::StaleCapabilityを返す。
    fn compare_and_transition(
        &self,
        resource_id: &str,
        expected_state: &str,
        expected_revision: u64,
        next_state: &str,
    ) -> Result<u64, TransitionError>;

    /// revision = 1で新規リソースを初期状態で作成する
    fn create(
        &self,
        resource_id: &str,
        initial_state: &str,
    ) -> Result<u64, TransitionError>;
}

impl<T: StateBackend> StateBackend for std::sync::Arc<T> {
    fn load(&self, resource_id: &str) -> Result<(String, u64), TransitionError> {
        (**self).load(resource_id)
    }
    fn compare_and_transition(&self, resource_id: &str, expected_state: &str, expected_revision: u64, next_state: &str) -> Result<u64, TransitionError> {
        (**self).compare_and_transition(resource_id, expected_state, expected_revision, next_state)
    }
    fn create(&self, resource_id: &str, initial_state: &str) -> Result<u64, TransitionError> {
        (**self).create(resource_id, initial_state)
    }
}

