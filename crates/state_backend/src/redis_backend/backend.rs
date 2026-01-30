use std::time::Duration;

use fred::prelude::*;

use crate::{
    backend::{BackendDecision, LimitSpec, StateBackend},
    error::BackendError,
    redis_backend::scripts::FIXED_WINDOW_LUA,
};

pub struct RedisBackend {
    client: Client,
    key_prefix: String,
}

impl RedisBackend {
    /// Create and connect a backend from a Redis URL.
    ///
    /// Example URL: redis://127.0.0.1:6379
    pub async fn connect(redis_url: &str) -> Result<Self, BackendError> {
        let config = Config::from_url(redis_url)?;
        let client = Client::new(config, None, None, None);

        // Start background connection tasks.
        client.connect();
        client.wait_for_connect().await?;

        Ok(Self {
            client,
            key_prefix: "rl".to_string(),
        })
    }

    /// Optional prefix override for key names.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    pub async fn connect_from_env() -> Result<Self, BackendError> {
        let url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self::connect(&url).await
    }

    fn redis_key(&self, namespace: &str, key: &str) -> String {
        format!("{}:{}:{}", self.key_prefix, namespace, key)
    }
}

#[async_trait::async_trait]
impl StateBackend for RedisBackend {
    async fn check(
        &self,
        namespace: &str,
        key: &str,
        limit: LimitSpec,
    ) -> Result<BackendDecision, BackendError> {
        let rk = self.redis_key(namespace, key);

        let window_ms: i64 = limit.window.as_millis().try_into().unwrap_or(i64::MAX);

        let max: i64 = limit.max as i64;

        // EVAL <script> <numkeys> <key> <argv...>
        // fred's eval signature is: eval(script, keys, args)
        let resp: Vec<i64> = self
            .client
            .eval(FIXED_WINDOW_LUA, vec![rk], vec![window_ms, max])
            .await?;

        if resp.len() != 2 {
            return Err(BackendError::InvalidResponse);
        }

        let allowed = resp[0];
        let retry_after_ms = resp[1];

        if allowed == 1 {
            Ok(BackendDecision::Allow)
        } else {
            Ok(BackendDecision::Deny {
                retry_after: Duration::from_millis(retry_after_ms.max(0) as u64),
            })
        }
    }
}
