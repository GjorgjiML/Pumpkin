//! ChunkyPMC - Chunky-style world pregeneration for Pumpkin.
//!
//! Commands:
//! - /chunky center
//! - /chunky center <x> <z>
//! - /chunky radius <chunks>
//! - /chunky start
//! - /chunky pause
//! - /chunky continue
//! - /chunky cancel
//! - /chunky progress

#![allow(improper_ctypes_definitions)]

use pumpkin::command::args::{Arg, ConsumedArgs, simple::SimpleArgConsumer};
use pumpkin::command::tree::CommandTree;
use pumpkin::command::tree::builder::{argument, literal, require};
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin::entity::player::Player;
use pumpkin::plugin::api::Context;
use pumpkin::plugin::{Plugin, PluginMetadata};
use pumpkin::world::World;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const ARG_X: &str = "x";
const ARG_Z: &str = "z";
const ARG_RADIUS: &str = "radius";
const DEFAULT_RADIUS: i32 = 128;
const CHUNKS_PER_STEP: usize = 32;
const MIN_RADIUS: i32 = 1;
const MAX_RADIUS: i32 = 8192;

#[unsafe(no_mangle)]
pub static PUMPKIN_API_VERSION: u32 = pumpkin::plugin::PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub static METADATA: PluginMetadata<'static> = PluginMetadata {
    name: "chunky_pmc",
    version: env!("CARGO_PKG_VERSION"),
    authors: "Pumpkin",
    description: "Chunky-style world pregeneration (/chunky)",
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin() -> Box<dyn Plugin> {
    Box::new(ChunkyPMCPlugin::new())
}

#[derive(Clone)]
struct ChunkySettings {
    center_chunk_x: i32,
    center_chunk_z: i32,
    radius: i32,
}

struct PregenTask {
    world_name: String,
    center_chunk_x: i32,
    center_chunk_z: i32,
    radius: i32,
    total_chunks: usize,
    generated_chunks: usize,
    paused: bool,
    cancelled: bool,
    finished: bool,
    started_at: Instant,
}

#[derive(Clone)]
struct ChunkyState {
    settings: Arc<Mutex<ChunkySettings>>,
    task: Arc<Mutex<Option<Arc<Mutex<PregenTask>>>>>,
}

impl ChunkyState {
    fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(ChunkySettings {
                center_chunk_x: 0,
                center_chunk_z: 0,
                radius: DEFAULT_RADIUS,
            })),
            task: Arc::new(Mutex::new(None)),
        }
    }
}

struct ChunkyPMCPlugin {
    state: ChunkyState,
}

impl ChunkyPMCPlugin {
    fn new() -> Self {
        Self {
            state: ChunkyState::new(),
        }
    }
}

fn build_positions(center_chunk_x: i32, center_chunk_z: i32, radius: i32) -> Vec<Vector2<i32>> {
    let side = (radius * 2 + 1) as usize;
    let mut out = Vec::with_capacity(side.saturating_mul(side));

    out.push(Vector2::new(center_chunk_x, center_chunk_z));
    for ring in 1..=radius {
        let min_x = center_chunk_x - ring;
        let max_x = center_chunk_x + ring;
        let min_z = center_chunk_z - ring;
        let max_z = center_chunk_z + ring;

        for x in min_x..=max_x {
            out.push(Vector2::new(x, min_z));
            out.push(Vector2::new(x, max_z));
        }
        for z in (min_z + 1)..=(max_z - 1) {
            out.push(Vector2::new(min_x, z));
            out.push(Vector2::new(max_x, z));
        }
    }

    out
}

fn format_duration(d: Duration) -> String {
    let total = d.as_secs();
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}h {m}m {s}s")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}

fn parse_i32(args: &ConsumedArgs<'_>, name: &str) -> Option<i32> {
    match args.get(name) {
        Some(Arg::Simple(s)) => s.parse::<i32>().ok(),
        _ => None,
    }
}

async fn send_task_finished_message(player: &Option<Arc<Player>>, cancelled: bool) {
    let msg = if cancelled {
        TextComponent::text("Chunky: task cancelled.").color_named(NamedColor::Yellow)
    } else {
        TextComponent::text("Chunky: pregeneration complete.").color_named(NamedColor::Green)
    };
    if let Some(player) = player {
        player.send_system_message(&msg).await;
    }
}

async fn run_pregen_task(
    task: Arc<Mutex<PregenTask>>,
    world: Arc<World>,
    positions: Vec<Vector2<i32>>,
    maybe_player: Option<Arc<Player>>,
) {
    let mut index = 0usize;
    while index < positions.len() {
        {
            let guard = task.lock().await;
            if guard.cancelled {
                drop(guard);
                world.force_save().await;
                let mut guard = task.lock().await;
                guard.finished = true;
                drop(guard);
                send_task_finished_message(&maybe_player, true).await;
                return;
            }
            if guard.paused {
                drop(guard);
                tokio::time::sleep(Duration::from_millis(250)).await;
                continue;
            }
        }

        let end = (index + CHUNKS_PER_STEP).min(positions.len());
        for pos in &positions[index..end] {
            world.level.get_chunk(pos.clone()).await;
        }

        {
            let mut guard = task.lock().await;
            guard.generated_chunks = end;
        }

        index = end;
        tokio::time::sleep(Duration::from_millis(2)).await;
    }

    world.force_save().await;
    {
        let mut guard = task.lock().await;
        guard.finished = true;
    }
    send_task_finished_message(&maybe_player, false).await;
}

struct SetCenterHereExecutor {
    state: ChunkyState,
}

impl CommandExecutor for SetCenterHereExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(pos) = sender.position() else {
                sender
                    .send_message(
                        TextComponent::text("Only players can use /chunky center without coordinates.")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let chunk_x = (pos.x.floor() as i32).div_euclid(16);
            let chunk_z = (pos.z.floor() as i32).div_euclid(16);
            let mut settings = self.state.settings.lock().await;
            settings.center_chunk_x = chunk_x;
            settings.center_chunk_z = chunk_z;

            sender
                .send_message(
                    TextComponent::text(format!(
                        "Chunky center set to chunk ({chunk_x}, {chunk_z}) from your position."
                    ))
                    .color_named(NamedColor::Green),
                )
                .await;
            Ok(1)
        })
    }
}

struct SetCenterCoordsExecutor {
    state: ChunkyState,
}

impl CommandExecutor for SetCenterCoordsExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(block_x) = parse_i32(args, ARG_X) else {
                sender
                    .send_message(
                        TextComponent::text("Usage: /chunky center <x> <z>")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };
            let Some(block_z) = parse_i32(args, ARG_Z) else {
                sender
                    .send_message(
                        TextComponent::text("Usage: /chunky center <x> <z>")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let chunk_x = block_x.div_euclid(16);
            let chunk_z = block_z.div_euclid(16);

            let mut settings = self.state.settings.lock().await;
            settings.center_chunk_x = chunk_x;
            settings.center_chunk_z = chunk_z;

            sender
                .send_message(
                    TextComponent::text(format!(
                        "Chunky center set to blocks ({block_x}, {block_z}) -> chunk ({chunk_x}, {chunk_z})."
                    ))
                    .color_named(NamedColor::Green),
                )
                .await;
            Ok(1)
        })
    }
}

struct RadiusExecutor {
    state: ChunkyState,
}

impl CommandExecutor for RadiusExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(radius) = parse_i32(args, ARG_RADIUS) else {
                sender
                    .send_message(
                        TextComponent::text("Usage: /chunky radius <chunks>")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };
            if !(MIN_RADIUS..=MAX_RADIUS).contains(&radius) {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "Radius must be between {MIN_RADIUS} and {MAX_RADIUS} chunks."
                        ))
                        .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }

            let mut settings = self.state.settings.lock().await;
            settings.radius = radius;
            let total = (radius as i64 * 2 + 1).pow(2);
            sender
                .send_message(
                    TextComponent::text(format!(
                        "Chunky radius set to {radius} chunks (~{total} chunks total)."
                    ))
                    .color_named(NamedColor::Green),
                )
                .await;
            Ok(1)
        })
    }
}

struct StartExecutor {
    state: ChunkyState,
}

impl CommandExecutor for StartExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(world) = sender.world() else {
                sender
                    .send_message(
                        TextComponent::text("Only players can start chunky pregeneration.")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let existing_task = { self.state.task.lock().await.clone() };
            if let Some(existing_task) = existing_task {
                let existing = existing_task.lock().await;
                if !existing.finished && !existing.cancelled {
                    sender
                        .send_message(
                            TextComponent::text("A chunky task is already running. Use /chunky progress.")
                                .color_named(NamedColor::Yellow),
                        )
                        .await;
                    return Ok(0);
                }
                drop(existing);

                let mut task_lock = self.state.task.lock().await;
                *task_lock = None;
            }

            let settings = self.state.settings.lock().await.clone();
            let positions =
                build_positions(settings.center_chunk_x, settings.center_chunk_z, settings.radius);
            let total_chunks = positions.len();
            let world_name = world.dimension.minecraft_name.to_string();

            let task = Arc::new(Mutex::new(PregenTask {
                world_name,
                center_chunk_x: settings.center_chunk_x,
                center_chunk_z: settings.center_chunk_z,
                radius: settings.radius,
                total_chunks,
                generated_chunks: 0,
                paused: false,
                cancelled: false,
                finished: false,
                started_at: Instant::now(),
            }));

            {
                let mut task_lock = self.state.task.lock().await;
                *task_lock = Some(task.clone());
            }

            let maybe_player = sender.as_player();
            tokio::spawn(run_pregen_task(task, world, positions, maybe_player));

            sender
                .send_message(
                    TextComponent::text(format!(
                        "Chunky started: center chunk ({}, {}), radius {} ({} chunks).",
                        settings.center_chunk_x, settings.center_chunk_z, settings.radius, total_chunks
                    ))
                    .color_named(NamedColor::Green),
                )
                .await;
            Ok(1)
        })
    }
}

struct PauseExecutor {
    state: ChunkyState,
}

impl CommandExecutor for PauseExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let task = self.state.task.lock().await.clone();
            let Some(task) = task else {
                sender
                    .send_message(
                        TextComponent::text("No chunky task is running.").color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let mut task = task.lock().await;
            if task.finished {
                sender
                    .send_message(
                        TextComponent::text("Task already finished. Use /chunky start for a new one.")
                            .color_named(NamedColor::Yellow),
                    )
                    .await;
                return Ok(0);
            }
            task.paused = true;
            sender
                .send_message(TextComponent::text("Chunky task paused.").color_named(NamedColor::Yellow))
                .await;
            Ok(1)
        })
    }
}

struct ContinueExecutor {
    state: ChunkyState,
}

impl CommandExecutor for ContinueExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let task = self.state.task.lock().await.clone();
            let Some(task) = task else {
                sender
                    .send_message(
                        TextComponent::text("No chunky task is running.").color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let mut task = task.lock().await;
            if task.finished {
                sender
                    .send_message(
                        TextComponent::text("Task already finished. Use /chunky start for a new one.")
                            .color_named(NamedColor::Yellow),
                    )
                    .await;
                return Ok(0);
            }
            task.paused = false;
            sender
                .send_message(TextComponent::text("Chunky task resumed.").color_named(NamedColor::Green))
                .await;
            Ok(1)
        })
    }
}

struct CancelExecutor {
    state: ChunkyState,
}

impl CommandExecutor for CancelExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let task = self.state.task.lock().await.clone();
            let Some(task) = task else {
                sender
                    .send_message(
                        TextComponent::text("No chunky task is running.").color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let mut task = task.lock().await;
            if task.finished {
                sender
                    .send_message(
                        TextComponent::text("Task already finished.").color_named(NamedColor::Yellow),
                    )
                    .await;
                return Ok(0);
            }

            task.cancelled = true;
            task.paused = false;
            sender
                .send_message(TextComponent::text("Chunky task cancellation requested.").color_named(NamedColor::Yellow))
                .await;
            Ok(1)
        })
    }
}

struct ProgressExecutor {
    state: ChunkyState,
}

impl CommandExecutor for ProgressExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let settings = self.state.settings.lock().await.clone();
            let task = self.state.task.lock().await.clone();
            let Some(task) = task else {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "No active chunky task. Current setup: center chunk ({}, {}), radius {}.",
                            settings.center_chunk_x, settings.center_chunk_z, settings.radius
                        ))
                        .color_named(NamedColor::Aqua),
                    )
                    .await;
                return Ok(1);
            };

            let task = task.lock().await;
            let total = task.total_chunks.max(1);
            let generated = task.generated_chunks;
            let pct = (generated as f64 / total as f64) * 100.0;
            let elapsed = task.started_at.elapsed();
            let speed = generated as f64 / elapsed.as_secs_f64().max(0.001);
            let remaining = task.total_chunks.saturating_sub(generated);
            let eta = if speed > 0.0 {
                Duration::from_secs_f64(remaining as f64 / speed)
            } else {
                Duration::from_secs(0)
            };

            let status = if task.finished && task.cancelled {
                "cancelled"
            } else if task.finished {
                "done"
            } else if task.paused {
                "paused"
            } else {
                "running"
            };

            sender
                .send_message(
                    TextComponent::text(format!(
                        "Chunky {status}: {generated}/{total} ({pct:.2}%), center ({}, {}), radius {}, world {}, elapsed {}, eta {}.",
                        task.center_chunk_x,
                        task.center_chunk_z,
                        task.radius,
                        task.world_name,
                        format_duration(elapsed),
                        format_duration(eta),
                    ))
                    .color_named(NamedColor::Aqua),
                )
                .await;
            Ok(1)
        })
    }
}

fn build_tree(state: ChunkyState) -> CommandTree {
    let op_only = require(|sender| sender.has_permission_lvl(PermissionLvl::Two));

    CommandTree::new(["chunky"], "Chunky-style pregeneration commands")
        .then(
            op_only
                .then(
                    literal("center")
                        .execute(SetCenterHereExecutor {
                            state: state.clone(),
                        })
                        .then(
                            argument(ARG_X, SimpleArgConsumer).then(
                                argument(ARG_Z, SimpleArgConsumer).execute(SetCenterCoordsExecutor {
                                    state: state.clone(),
                                }),
                            ),
                        ),
                )
                .then(
                    literal("radius").then(argument(ARG_RADIUS, SimpleArgConsumer).execute(
                        RadiusExecutor {
                            state: state.clone(),
                        },
                    )),
                )
                .then(literal("start").execute(StartExecutor {
                    state: state.clone(),
                }))
                .then(literal("pause").execute(PauseExecutor {
                    state: state.clone(),
                }))
                .then(literal("continue").execute(ContinueExecutor {
                    state: state.clone(),
                }))
                .then(literal("cancel").execute(CancelExecutor {
                    state: state.clone(),
                }))
                .then(literal("progress").execute(ProgressExecutor { state })),
        )
}

impl Plugin for ChunkyPMCPlugin {
    fn on_load(
        &mut self,
        context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        let state = self.state.clone();
        Box::pin(async move {
            context
                .register_permission(pumpkin_util::permission::Permission::new(
                    "chunky_pmc:admin",
                    "Use chunky pregeneration commands",
                    pumpkin_util::permission::PermissionDefault::Op(PermissionLvl::Two),
                ))
                .await
                .ok();

            let tree = build_tree(state);
            context.register_command(tree, "chunky_pmc:admin").await;
            log::info!("chunky_pmc: Loaded (/chunky)");
            Ok(())
        })
    }

    fn on_unload(
        &mut self,
        _context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        let state = self.state.clone();
        Box::pin(async move {
            if let Some(task) = state.task.lock().await.clone() {
                let mut task = task.lock().await;
                task.cancelled = true;
                task.paused = false;
            }
            Ok(())
        })
    }
}
