//! Shared plugin state (pool, metrics, runtime).

use sqlx::PgPool;
use std::sync::{Arc, RwLock};

/// Shared reference passed to command executors and event handlers.
#[derive(Clone)]
pub struct PluginState {
    pub runtime: Arc<tokio::runtime::Runtime>,
    pub db_pool: Arc<RwLock<Option<PgPool>>>,
    pub db_latency_ms: Arc<RwLock<Vec<f64>>>,
    pub last_error: Arc<RwLock<Option<String>>>,
}

impl PluginState {
    /// Run async DB work on the plugin's tokio runtime.
    #[inline]
    pub fn block_on<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.runtime.block_on(f)
    }

    pub fn record_latency(&self, ms: f64) {
        if let Ok(mut latencies) = self.db_latency_ms.write() {
            latencies.push(ms);
            if latencies.len() > 100 {
                latencies.remove(0);
            }
        }
    }

    pub fn record_error(&self, err: &str) {
        if let Ok(mut last_err) = self.last_error.write() {
            *last_err = Some(err.to_string());
        }
    }
}
