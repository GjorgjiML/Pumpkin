//! Command executors for each /wg subcommand.

use std::collections::HashSet;

use pumpkin::command::{CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs};
use pumpkin::command::args::Arg;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::commands::{ARG_FLAG, ARG_ID, ARG_PLAYER, ARG_VALUE};
use crate::handlers::WordGuardRef;
use crate::region::{Region, RegionFlags};

pub struct DefineExecutor(pub WordGuardRef);

impl CommandExecutor for DefineExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = match args.get(ARG_ID) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Usage: /wg define <id>")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let player = match sender.as_player() {
                Some(p) => p,
                None => {
                    sender
                        .send_message(
                            TextComponent::text("Only players can define regions")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };

            let dimension_id = player.living_entity.entity.world.load().dimension.id;
            let uuid = player.gameprofile.id;

            let (pos1, pos2) = {
                let sel = self.0.selections.read().await;
                match sel.get(&uuid) {
                    Some(p) => p,
                    None => {
                        sender
                            .send_message(
                                TextComponent::text(
                                    "Set two corners with the wand (stick): left-click pos1, right-click pos2",
                                )
                                .color_named(NamedColor::Yellow),
                            )
                            .await;
                        return Ok(0);
                    }
                }
            };

            let mut owners = HashSet::new();
            owners.insert(uuid);

            let region = Region {
                min: pos1,
                max: pos2,
                owners,
                members: HashSet::new(),
                flags: RegionFlags::all_protected(),
            };

            let mut regions = self.0.regions.write().await;
            if !regions.add(dimension_id, name.to_string(), region) {
                sender
                    .send_message(
                        TextComponent::text(format!("Region '{name}' already exists; remove it first"))
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }

            sender
                .send_message(
                    TextComponent::text(format!("Region '{name}' defined. You are the owner."))
                        .color_named(NamedColor::Green),
                )
                .await;
            let _ = server;
            Ok(1)
        })
    }
}

pub struct RemoveExecutor(pub WordGuardRef);

impl CommandExecutor for RemoveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = match args.get(ARG_ID) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Usage: /wg remove <id>")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let player = match sender.as_player() {
                Some(p) => p,
                None => {
                    sender
                        .send_message(
                            TextComponent::text("Only players can remove regions")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let dimension_id = player.living_entity.entity.world.load().dimension.id;

            let mut regions = self.0.regions.write().await;
            if regions.remove(dimension_id, name).is_some() {
                sender
                    .send_message(
                        TextComponent::text(format!("Region '{name}' removed"))
                            .color_named(NamedColor::Green),
                    )
                    .await;
            } else {
                sender
                    .send_message(
                        TextComponent::text(format!("Region '{name}' not found"))
                            .color_named(NamedColor::Red),
                    )
                    .await;
            }
            let _ = server;
            Ok(1)
        })
    }
}

pub struct AddOwnerExecutor(pub WordGuardRef);

impl CommandExecutor for AddOwnerExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = match args.get(ARG_ID) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Usage: /wg addowner <id> <player>")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let target = match args.get(ARG_PLAYER) {
                Some(Arg::Players(players)) => players.first().cloned(),
                _ => None,
            };
            let Some(target) = target else {
                sender
                    .send_message(
                        TextComponent::text("Specify one player")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };
            let player = match sender.as_player() {
                Some(p) => p,
                None => {
                    sender
                        .send_message(
                            TextComponent::text("Only players can add owners")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let dimension_id = player.living_entity.entity.world.load().dimension.id;

            let mut regions = self.0.regions.write().await;
            let Some(region) = regions.get_region_mut(dimension_id, name) else {
                sender
                    .send_message(
                        TextComponent::text(format!("Region '{name}' not found"))
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };
            if !region.owners.contains(&player.gameprofile.id) {
                sender
                    .send_message(
                        TextComponent::text("Only owners can add owners")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }
            region.owners.insert(target.gameprofile.id);
            region.members.remove(&target.gameprofile.id);
            sender
                .send_message(
                    TextComponent::text(format!("Added {} as owner", target.gameprofile.name))
                        .color_named(NamedColor::Green),
                )
                .await;
            let _ = server;
            Ok(1)
        })
    }
}

pub struct AddMemberExecutor(pub WordGuardRef);

impl CommandExecutor for AddMemberExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = match args.get(ARG_ID) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Usage: /wg addmember <id> <player>")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let target = match args.get(ARG_PLAYER) {
                Some(Arg::Players(players)) => players.first().cloned(),
                _ => None,
            };
            let Some(target) = target else {
                sender
                    .send_message(
                        TextComponent::text("Specify one player")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };
            let player = match sender.as_player() {
                Some(p) => p,
                None => {
                    sender
                        .send_message(
                            TextComponent::text("Only players can add members")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let dimension_id = player.living_entity.entity.world.load().dimension.id;

            let mut regions = self.0.regions.write().await;
            let Some(region) = regions.get_region_mut(dimension_id, name) else {
                sender
                    .send_message(
                        TextComponent::text(format!("Region '{name}' not found"))
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };
            if !region.owners.contains(&player.gameprofile.id) {
                sender
                    .send_message(
                        TextComponent::text("Only owners can add members")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }
            region.members.insert(target.gameprofile.id);
            sender
                .send_message(
                    TextComponent::text(format!("Added {} as member", target.gameprofile.name))
                        .color_named(NamedColor::Green),
                )
                .await;
            let _ = server;
            Ok(1)
        })
    }
}

pub struct FlagExecutor(pub WordGuardRef);

impl CommandExecutor for FlagExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = match args.get(ARG_ID) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text(
                                "Usage: /wg flag <id> <block-break|block-place> <allow|deny>",
                            )
                            .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let flag_name = match args.get(ARG_FLAG) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text(
                                "Usage: /wg flag <id> <block-break|block-place> <allow|deny>",
                            )
                            .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let value = match args.get(ARG_VALUE) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text(
                                "Use allow (everyone) or deny (only owners/members)",
                            )
                            .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let protect = match value {
                "deny" => true,
                "allow" => false,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Value must be 'allow' or 'deny'")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let player = match sender.as_player() {
                Some(p) => p,
                None => {
                    sender
                        .send_message(
                            TextComponent::text("Only players can set flags")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let dimension_id = player.living_entity.entity.world.load().dimension.id;

            let mut regions = self.0.regions.write().await;
            let Some(region) = regions.get_region_mut(dimension_id, name) else {
                sender
                    .send_message(
                        TextComponent::text(format!("Region '{name}' not found"))
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };
            if !region.owners.contains(&player.gameprofile.id) {
                sender
                    .send_message(
                        TextComponent::text("Only owners can set flags")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }
            match flag_name {
                "block-break" => region.flags.block_break = protect,
                "block-place" => region.flags.block_place = protect,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Flag must be 'block-break' or 'block-place'")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            }
            let val_str = if protect { "deny" } else { "allow" };
            sender
                .send_message(
                    TextComponent::text(format!(
                        "Region '{name}': {} set to {}",
                        flag_name, val_str,
                    ))
                    .color_named(NamedColor::Green),
                )
                .await;
            let _ = server;
            Ok(1)
        })
    }
}

pub struct ListExecutor(pub WordGuardRef);

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let player = match sender.as_player() {
                Some(p) => p,
                None => {
                    sender
                        .send_message(
                            TextComponent::text("Only players can list regions")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };
            let dimension_id = player.living_entity.entity.world.load().dimension.id;
            let regions = self.0.regions.read().await;
            let list = regions.list(dimension_id);
            if list.is_empty() {
                sender
                    .send_message(
                        TextComponent::text("No regions in this dimension")
                            .color_named(NamedColor::Yellow),
                    )
                    .await;
            } else {
                let names: Vec<_> = list.into_iter().map(|(n, _, _)| n).collect();
                sender
                    .send_message(
                        TextComponent::text(format!("Regions: {}", names.join(", ")))
                            .color_named(NamedColor::Aqua),
                    )
                    .await;
            }
            let _ = server;
            let _ = args;
            Ok(1)
        })
    }
}

pub struct WandExecutor(pub WordGuardRef);

impl CommandExecutor for WandExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            if sender.as_player().is_none() {
                sender
                    .send_message(
                        TextComponent::text("Only players can use the wand")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }
            sender
                .send_message(
                    TextComponent::text(
                        "Use a stick: left-click = pos1, right-click block = pos2. Then /wg define <id>",
                    )
                    .color_named(NamedColor::Green),
                )
                .await;
            let _ = server;
            Ok(1)
        })
    }
}
