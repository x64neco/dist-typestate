/// 状態遷移で発生し得るエラー
///
/// StaleCapabilityは異常ではなく正常な競合検出として扱う。
#[derive(Debug)]
pub enum TransitionError {
    /// revision不一致(他のノード/プロセスが先に遷移した)
    StaleCapability {
        expected_revision: u64,
        actual_revision: Option<u64>,
    },

    /// リソースが見つからない
    NotFound { id: String },

    /// create時にリソースが既に存在する
    AlreadyExists { id: String },

    /// バックエンドのI/Oエラー
    BackendError(String),
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleCapability {
                expected_revision,
                actual_revision,
            } => {
                write!(f, "stale capability: expected revision {expected_revision}")?;
                if let Some(actual) = actual_revision {
                    write!(f, ", actual {actual}")?;
                }
                Ok(())
            }
            Self::NotFound { id } => write!(f, "resource not found: {id}"),
            Self::AlreadyExists { id } => write!(f, "resource already exists: {id}"),
            Self::BackendError(msg) => write!(f, "backend error: {msg}"),
        }
    }
}

impl std::error::Error for TransitionError {}
