//! Handler for PlayerAttackEvent — blocks PvP in non-PvP zones.

use crate::state::PluginState;
use pumpkin::plugin::EventHandler;
use pumpkin::plugin::api::events::player::player_attack::PlayerAttackEvent;
use pumpkin::server::Server;
use std::pin::Pin;
use std::sync::Arc;

pub struct ZonePvpHandler {
    pub state: PluginState,
}

impl EventHandler<PlayerAttackEvent> for ZonePvpHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerAttackEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let attacker_pos = event.attacker.position();
            let victim_pos = event.victim.position();

            let attacker_zone =
                self.state
                    .zone_engine
                    .zone_at(attacker_pos.x, attacker_pos.y, attacker_pos.z);
            let victim_zone =
                self.state
                    .zone_engine
                    .zone_at(victim_pos.x, victim_pos.y, victim_pos.z);

            // If either side is in a no-PvP zone, deny the attack.
            if !attacker_zone.pvp_enabled || !victim_zone.pvp_enabled {
                event.cancelled = true;
                log::info!(
                    "albion_zones: blocked PvP {} -> {} (attacker zone: {}, victim zone: {})",
                    event.attacker.gameprofile.name,
                    event.victim.gameprofile.name,
                    attacker_zone.name,
                    victim_zone.name
                );
            }
        })
    }
}
