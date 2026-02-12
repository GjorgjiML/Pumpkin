//! Handler for PlayerDeathEvent — sets drop_percent and trash_percent from zone rules.

use crate::config::DeathRule;
use crate::state::PluginState;
use pumpkin::plugin::EventHandler;
use pumpkin::plugin::api::events::player::player_death::PlayerDeathEvent;
use pumpkin::server::Server;
use std::pin::Pin;
use std::sync::Arc;

pub struct ZoneDeathHandler {
    pub state: PluginState,
}

impl EventHandler<PlayerDeathEvent> for ZoneDeathHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerDeathEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let pos = event.position;
            let lookup = self.state.zone_engine.zone_at(pos.x, pos.y, pos.z);

            let (drop_percent, trash_percent) = match lookup.death_rule {
                DeathRule::Safe => (0, 0),
                DeathRule::Partial => (lookup.partial_drop_percent, 0),
                DeathRule::FullLoot => (100, self.state.zone_engine.trash_chance_percent),
            };

            event.drop_percent = drop_percent;
            event.trash_percent = trash_percent;

            log::info!(
                "albion_zones: {} died in {} — drop {}%, trash {}%",
                event.player.gameprofile.name,
                lookup.name,
                drop_percent,
                trash_percent,
            );
        })
    }
}
