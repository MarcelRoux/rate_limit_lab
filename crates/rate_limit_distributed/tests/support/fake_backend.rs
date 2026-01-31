use async_trait::async_trait;
use state_backend::{BackendDecision, BackendError, LimitSpec, StateBackend};

#[derive(Clone)]
pub(crate) struct FakeBackend {
    pub(crate) decision: Result<BackendDecision, ()>,
}

#[async_trait]
impl StateBackend for FakeBackend {
    async fn check(
        &self,
        _namespace: &str,
        _key: &str,
        _limit: LimitSpec,
    ) -> Result<BackendDecision, BackendError> {
        self.decision
            .clone()
            .map_err(|_| BackendError::InvalidResponse)
    }
}
