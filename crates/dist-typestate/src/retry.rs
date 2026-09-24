use crate::error::TransitionError;

/// リトライ設定
pub struct RetryPolicy {
    pub max_attempts: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 3 }
    }
}

/// StaleCapability時にreload&retryするヘルパー関数
///
/// load_fn: backendから最新状態をロードして型付きリソースを取得する関数
/// action_fn: ロードされたリソースに対して遷移操作を行う関数
///
/// StaleCapabilityの場合だけリトライする。他のエラーは即座に返す。
pub fn retry_on_stale<T, R, E>(
    policy: &RetryPolicy,
    mut load_fn: impl FnMut() -> Result<T, TransitionError>,
    mut action_fn: impl FnMut(T) -> Result<R, TransitionError>,
) -> Result<R, TransitionError> {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let resource = load_fn()?;
        match action_fn(resource) {
            Ok(result) => return Ok(result),
            Err(TransitionError::StaleCapability { .. }) if attempts < policy.max_attempts => {
                // retry
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
