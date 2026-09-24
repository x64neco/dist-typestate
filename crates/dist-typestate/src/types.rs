/// Capabilityの鮮度
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Freshness {
    /// ローカルのrevisionがリモートと一致している
    Current,
    /// ローカルのrevisionがリモートより古い
    Stale {
        local_revision: u64,
        remote_revision: u64,
    },
}

/// health_checkの結果
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub resource_id: String,
    pub current_state: String,
    pub current_revision: u64,
    pub freshness: Freshness,
}
