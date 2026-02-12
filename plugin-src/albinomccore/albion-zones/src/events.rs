//! Event handlers for zone transitions, player join/leave, and movement.

use crate::bossbar;
use crate::config::RiskLevel;
use crate::newbie;
use crate::state::{PlayerZoneState, PluginState};
use pumpkin::entity::player::TitleMode;
use pumpkin::plugin::EventHandler;
use pumpkin::plugin::api::events::player::player_join::PlayerJoinEvent;
use pumpkin::plugin::api::events::player::player_leave::PlayerLeaveEvent;
use pumpkin::plugin::api::events::player::player_move::PlayerMoveEvent;
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use std::pin::Pin;
use std::sync::Arc;

// ───────────────────────────── Player Join ─────────────────────────────

pub struct ZoneJoinHandler {
    pub state: PluginState,
}

impl EventHandler<PlayerJoinEvent> for ZoneJoinHandler {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a PlayerJoinEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let player = &event.player;
            let uuid = player.gameprofile.id;
            let pos = player.position();
            let engine = &self.state.zone_engine;

            let lookup = engine.zone_at(pos.x, pos.y, pos.z);

            // Send initial bossbar
            let bossbar_uuid =
                bossbar::send_zone_bossbar(player, &lookup.name, lookup.risk).await;

            // Record player zone state
            self.state.set_player_zone(
                uuid,
                PlayerZoneState {
                    current_zone_name: lookup.name.clone(),
                    current_risk: lookup.risk,
                    bossbar_uuid,
                    danger_confirmed: false,
                },
            );

            // Show welcome action bar
            let welcome = format!("Entered {} — {}", lookup.name, lookup.risk.label());
            player
                .show_title(
                    &TextComponent::text(welcome)
                        .color_named(bossbar::risk_to_text_color(lookup.risk)),
                    &TitleMode::ActionBar,
                )
                .await;

            log::info!(
                "albion_zones: {} joined in {} ({})",
                player.gameprofile.name,
                lookup.name,
                lookup.risk,
            );
        })
    }
}

// ───────────────────────────── Player Leave ─────────────────────────────

pub struct ZoneLeaveHandler {
    pub state: PluginState,
}

impl EventHandler<PlayerLeaveEvent> for ZoneLeaveHandler {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a PlayerLeaveEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let uuid = event.player.gameprofile.id;
            if let Some(pz) = self.state.remove_player(&uuid) {
                bossbar::remove_zone_bossbar(&event.player, pz.bossbar_uuid).await;
            }
            log::info!(
                "albion_zones: {} left, cleaned up zone state",
                event.player.gameprofile.name
            );
        })
    }
}

// ───────────────────────────── Player Move ─────────────────────────────

pub struct ZoneMoveHandler {
    pub state: PluginState,
}

impl EventHandler<PlayerMoveEvent> for ZoneMoveHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerMoveEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let player = &event.player;
            let uuid = player.gameprofile.id;
            let engine = &self.state.zone_engine;

            let old_zone = engine.zone_name_at(event.from.x, event.from.y, event.from.z);
            let new_lookup = engine.zone_at(event.to.x, event.to.y, event.to.z);

            // No zone change — skip
            if old_zone == new_lookup.name {
                return;
            }

            // ── Newbie protection: block entry to dangerous zones ──
            if new_lookup.risk.is_dangerous() {
                let blocked = self.state.block_on(async {
                    newbie::check_newbie_block(&self.state, player, new_lookup.risk).await
                });
                if blocked {
                    event.cancelled = true;

                    player
                        .show_title(
                            &TextComponent::text(format!(
                                "You cannot enter {} zones yet!",
                                new_lookup.risk
                            ))
                            .color_named(NamedColor::Red),
                            &TitleMode::ActionBar,
                        )
                        .await;
                    return;
                }
            }

            // ── Zone transition: update bossbar + notify ──
            let pz = self.state.get_player_zone(&uuid);
            if let Some(pz) = &pz {
                bossbar::update_zone_bossbar(
                    player,
                    &pz.bossbar_uuid,
                    &new_lookup.name,
                    new_lookup.risk,
                )
                .await;
            }

            // Show transition title
            player
                .show_title(
                    &TextComponent::text(format!("Entering: {}", new_lookup.name))
                        .color_named(bossbar::risk_to_text_color(new_lookup.risk)),
                    &TitleMode::Title,
                )
                .await;

            // Show risk subtitle
            let subtitle_text = match new_lookup.risk {
                RiskLevel::Green => "Safe Zone — No PvP, No Item Loss",
                RiskLevel::Yellow => "Caution — PvP Enabled, Partial Loot on Death",
                RiskLevel::Red => "DANGER — Full Loot PvP! Items WILL be lost!",
                RiskLevel::Black => "LETHAL — Full Loot PvP! Maximum Risk!",
            };
            player
                .show_title(
                    &TextComponent::text(subtitle_text)
                        .color_named(bossbar::risk_to_text_color(new_lookup.risk)),
                    &TitleMode::SubTitle,
                )
                .await;

            player.send_title_animation(10, 60, 20).await;

            // Update tracked state
            let (bossbar_uuid, was_confirmed) = pz
                .as_ref()
                .map_or((uuid::Uuid::new_v4(), false), |p| {
                    (p.bossbar_uuid, p.danger_confirmed)
                });
            self.state.set_player_zone(
                uuid,
                PlayerZoneState {
                    current_zone_name: new_lookup.name.clone(),
                    current_risk: new_lookup.risk,
                    bossbar_uuid,
                    danger_confirmed: was_confirmed,
                },
            );

            log::info!(
                "albion_zones: {} crossed into {} ({old_zone} -> {})",
                player.gameprofile.name,
                new_lookup.name,
                new_lookup.risk,
            );
        })
    }
}
