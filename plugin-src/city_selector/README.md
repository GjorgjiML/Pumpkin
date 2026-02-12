# City Selector

Pumpkin plugin for the **lobby** server: compass menu to travel to city servers via **Velocity** (or BungeeCord).

## Features

- **Compass in lobby**: Right-click the compass to open an inventory GUI selector.
- **Four cities**: **Ratsku**, **Vatsku**, **Ratuskuu**, **AJkaz** (each with zones outside the city).
- **Proxy transfer**: Uses BungeeCord plugin message `Connect` so Velocity/BungeeCord moves the player to the selected server.

## Setup

1. **Velocity** (or BungeeCord): Configure servers `lobby`, `ratsku`, `vatsku`, `ratuskuu`, `ajkaz` in `config/velocity.toml` (see `VELOCITY_INTERNAL_NETWORK.md` for 10.0.0.2 internal network).
2. **Pumpkin lobby**: Enable proxy in `config/features.toml` and set the Velocity forwarding secret.
3. **Plugin**: Copy `libcity_selector.so` to the lobby server’s `plugins/` folder.
4. **Config**: After first run, edit `plugin-data/city_selector/config.toml` to match your Velocity server names if they differ.

## Config (`plugin-data/city_selector/config.toml`)

```toml
[lobby]
give_compass_on_join = true

[servers]
ratsku = "ratsku"
vatsku = "vatsku"
ratuskuu = "ratuskuu"
ajkaz = "ajkaz"
```

Server values must match the server IDs in Velocity’s `[servers]` section.

## Build

From the Pumpkin workspace root:

```bash
cargo build -p city_selector --release
```

Output: `target/release/libcity_selector.so` (use on Linux; rename/copy as needed for your setup).
