//! Bossbar management — show/update/remove zone bossbars for players.

use crate::config::RiskLevel;
use pumpkin::entity::player::Player;
use pumpkin::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use uuid::Uuid;

/// Map risk level to bossbar color.
#[must_use]
pub const fn risk_to_bossbar_color(risk: RiskLevel) -> BossbarColor {
    match risk {
        RiskLevel::Green => BossbarColor::Green,
        RiskLevel::Yellow => BossbarColor::Yellow,
        RiskLevel::Red => BossbarColor::Red,
        RiskLevel::Black => BossbarColor::Purple,
    }
}

/// Map risk level to text color for the bossbar title.
#[must_use]
pub const fn risk_to_text_color(risk: RiskLevel) -> NamedColor {
    match risk {
        RiskLevel::Green => NamedColor::Green,
        RiskLevel::Yellow => NamedColor::Yellow,
        RiskLevel::Red => NamedColor::Red,
        RiskLevel::Black => NamedColor::DarkPurple,
    }
}

/// Build the bossbar title text component.
#[must_use]
pub fn zone_bossbar_title(zone_name: &str, risk: RiskLevel) -> TextComponent {
    let pvp_tag = if risk == RiskLevel::Green {
        " [PvP OFF]"
    } else {
        " [PvP ON]"
    };
    let label = format!("{zone_name} — {}{pvp_tag}", risk.label());
    TextComponent::text(label).color_named(risk_to_text_color(risk))
}

/// Create a fresh bossbar for a zone.
#[must_use]
pub fn create_zone_bossbar(zone_name: &str, risk: RiskLevel) -> Bossbar {
    let title = zone_bossbar_title(zone_name, risk);
    let mut bar = Bossbar::new(title);
    bar.health = 1.0; // full bar
    bar.color = risk_to_bossbar_color(risk);
    bar.division = BossbarDivisions::NoDivision;
    bar.flags = BossbarFlags::NoFlags;
    bar
}

/// Send a new bossbar to a player. Returns the bossbar UUID for tracking.
pub async fn send_zone_bossbar(player: &Player, zone_name: &str, risk: RiskLevel) -> Uuid {
    let bar = create_zone_bossbar(zone_name, risk);
    let uuid = bar.uuid;
    player.send_bossbar(&bar).await;
    uuid
}

/// Update an existing bossbar to a new zone.
pub async fn update_zone_bossbar(
    player: &Player,
    bossbar_uuid: &Uuid,
    zone_name: &str,
    risk: RiskLevel,
) {
    let title = zone_bossbar_title(zone_name, risk);
    player.update_bossbar_title(bossbar_uuid, title).await;
    player
        .update_bossbar_style(
            bossbar_uuid,
            risk_to_bossbar_color(risk),
            BossbarDivisions::NoDivision,
        )
        .await;
}

/// Remove a bossbar from a player.
pub async fn remove_zone_bossbar(player: &Player, bossbar_uuid: Uuid) {
    player.remove_bossbar(bossbar_uuid).await;
}
