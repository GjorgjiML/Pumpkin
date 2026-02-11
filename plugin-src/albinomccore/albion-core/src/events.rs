//! Event handlers.

use super::state::PluginState;
use albion_types::PlayerProfile;
use pumpkin::plugin::EventHandler;
use pumpkin::plugin::api::events::player::player_join::PlayerJoinEvent;
use std::pin::Pin;
use std::sync::Arc;

pub struct PlayerJoinHandler {
    pub state: PluginState,
}

impl EventHandler<PlayerJoinEvent> for PlayerJoinHandler {
    fn handle<'a>(
        &'a self,
        server: &'a Arc<pumpkin::server::Server>,
        event: &'a PlayerJoinEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let uuid = event.player.gameprofile.id;
            let name = &event.player.gameprofile.name;

            let pool = {
                let guard = self.state.db_pool.read().unwrap();
                guard.as_ref().cloned()
            };
            let Some(pool) = pool else {
                log::warn!("albion_core: DB pool not available on player join for {name}");
                return;
            };

            let result = self.state.block_on(async {
                let mut conn = pool.acquire().await.map_err(|e| format!("acquire: {e}"))?;

                let existing = sqlx::query_scalar::<_, i64>(
                    "SELECT 1 FROM albion_profiles WHERE uuid = $1",
                )
                .bind(uuid)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|e| format!("select: {e}"))?;

                if existing.is_some() {
                    return Ok::<bool, String>(false);
                }

                let profile = PlayerProfile::new(uuid);
                let mastery = serde_json::to_value(&profile.mastery).unwrap_or_default();
                let flags = serde_json::to_value(&profile.flags).unwrap_or_default();

                sqlx::query(
                    "INSERT INTO albion_profiles (uuid, silver, fame, mastery, flags) \
                     VALUES ($1, $2, $3, $4, $5) ON CONFLICT (uuid) DO NOTHING",
                )
                .bind(profile.uuid)
                .bind(profile.silver)
                .bind(profile.fame)
                .bind(&mastery)
                .bind(&flags)
                .execute(&mut *conn)
                .await
                .map_err(|e| format!("insert: {e}"))?;

                Ok(true)
            });

            match result {
                Ok(true) => log::info!("albion_core: Created profile for player {name}"),
                Ok(false) => log::info!("albion_core: Profile already exists for {name}"),
                Err(e) => {
                    log::error!("albion_core: Failed to handle join for {name}: {e}");
                    self.state.record_error(&e);
                }
            }

            let _ = server;
        })
    }
}
