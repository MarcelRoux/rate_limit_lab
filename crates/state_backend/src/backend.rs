use std::time::Duration;

use async_trait::async_trait;

use crate::error::BackendError;

/// Backend-agnostic rate limit spec.
///
/// For M3.1 we intentionally keep this simple:
/// - fixed window duration
/// - max allowed in the window
#[derive(Clone, Copy, Debug)]
pub struct LimitSpec {
    pub window: Duration,
    pub max: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendDecision {
    Allow,
    Deny { retry_after: Duration },
}

#[async_trait]
pub trait StateBackend: Send + Sync {
    async fn check(
        &self,
        namespace: &str,
        key: &str,
        limit: LimitSpec,
    ) -> Result<BackendDecision, BackendError>;
}
