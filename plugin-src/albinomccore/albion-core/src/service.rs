//! Plugin services exposed to other plugins.

use super::state::PluginState;
use sqlx::PgPool;
use std::sync::{Arc, RwLock};

/// Profile service — can be queried by other plugins.
pub struct ProfileService {
    pub db_pool: Arc<RwLock<Option<PgPool>>>,
    pub db_latency_ms: Arc<RwLock<Vec<f64>>>,
    pub last_error: Arc<RwLock<Option<String>>>,
}

impl From<&PluginState> for ProfileService {
    fn from(state: &PluginState) -> Self {
        Self {
            db_pool: Arc::clone(&state.db_pool),
            db_latency_ms: Arc::clone(&state.db_latency_ms),
            last_error: Arc::clone(&state.last_error),
        }
    }
}

impl pumpkin::plugin::api::Payload for ProfileService {
    fn get_name_static() -> &'static str {
        "albion_core::ProfileService"
    }
    fn get_name(&self) -> &'static str {
        Self::get_name_static()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
