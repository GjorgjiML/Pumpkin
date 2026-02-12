//! Newbie protection — prevent new players from entering dangerous zones.
//!
//! Uses `albion_profiles.created_at` from albion_core to calculate play-time.
//! Hours-since-creation is computed in SQL to avoid pulling in datetime crates.

use crate::config::RiskLevel;
use crate::state::PluginState;
use pumpkin::entity::player::Player;

/// Check whether a player should be blocked from entering a dangerous zone.
///
/// Returns `true` if the player is blocked (too new), `false` if they may pass.
pub async fn check_newbie_block(
    state: &PluginState,
    player: &Player,
    _target_risk: RiskLevel,
) -> bool {
    let uuid = player.gameprofile.id;
    let required_hours = state.zone_engine.newbie_required_hours;

    // If required hours is 0, newbie protection is disabled
    if required_hours == 0 {
        return false;
    }

    // If this player already confirmed dangerous entry this session, let them pass
    if let Some(pz) = state.get_player_zone(&uuid) {
        if pz.danger_confirmed {
            return false;
        }
    }

    // Query profile creation time from albion_core's table
    let pool = {
        let guard = state.db_pool.read().unwrap();
        guard.as_ref().cloned()
    };
    let Some(pool) = pool else {
        // Can't verify — fail open (allow entry) rather than blocking due to DB issues
        log::warn!("albion_zones: DB not available for newbie check, allowing entry");
        return false;
    };

    // Compute hours elapsed since profile creation in SQL
    let result = sqlx::query_scalar::<_, f64>(
        "SELECT EXTRACT(EPOCH FROM NOW() - created_at) / 3600.0 \
         FROM albion_profiles WHERE uuid = $1",
    )
    .bind(uuid)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(elapsed_hours)) => {
            if elapsed_hours < required_hours as f64 {
                let remaining = required_hours as f64 - elapsed_hours;
                log::info!(
                    "albion_zones: Blocking {} from dangerous zone — {remaining:.1}h remaining",
                    player.gameprofile.name,
                );
                return true;
            }
            // Player is old enough — mark confirmed so we don't re-query
            state.set_danger_confirmed(&uuid);
            false
        }
        Ok(None) => {
            // No profile — block entry (they just joined, must be new)
            log::info!(
                "albion_zones: Blocking {} from dangerous zone — no profile found",
                player.gameprofile.name,
            );
            true
        }
        Err(e) => {
            log::error!("albion_zones: Newbie check failed: {e}");
            // Fail open
            false
        }
    }
}
