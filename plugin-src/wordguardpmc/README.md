# WordGuardPMC (WorldGuard for Pumpkin)

WorldGuard-style region protection for the [Pumpkin](https://pumpkinmc.org/) Minecraft server (Rust). Define cuboid regions, restrict block break/place to owners and members, and use flags per region.

**Repository:** [github.com/GjorgjiML/worldguardalbino](https://github.com/GjorgjiML/worldguardalbino)

---

## Features

- **Regions** — Define cuboid regions per dimension (Overworld, Nether, End).
- **Protection** — Only region owners and members can break (and when supported, place) blocks; everyone else is blocked.
- **Flags** — Per-region flags: `block-break` and `block-place` with `allow` / `deny`.
- **Selection wand** — Use a **stick**: left-click = pos1, right-click = pos2; chat shows the selected position.
- **OP-only commands** — All `/wg` commands require OP (permission level ≥ 1).

---

## Commands

| Command | Description |
|--------|-------------|
| `/wg define <id>` | Create a region from your current wand selection; you become the owner. |
| `/wg remove <id>` | Delete a region (same dimension). |
| `/wg addowner <id> <player>` | Add an owner (only existing owners). |
| `/wg addmember <id> <player>` | Add a member (only owners). |
| `/wg flag <id> <block-break\|block-place> <allow\|deny>` | Set region flag: `deny` = only owners/members, `allow` = everyone. |
| `/wg list` | List region names in your current dimension. |
| `/wg wand` | Show wand usage (stick: left = pos1, right = pos2). |

Alias: `/wordguard` for all of the above.

---

## Permissions

| Node | Default | Description |
|------|---------|-------------|
| `wordguardpmc:admin` | OP 2 | Use /wg commands (and OP check). |
| `wordguardpmc:bypass` | OP 4 | Bypass all region protection. |

Only **OP players** can use `/wg`; non-OP get “You do not have permission to perform this command.”

---

## Project layout (decentralized)

The plugin is split into focused modules instead of a single monolithic file:

```
src/
├── lib.rs          # Plugin entry, on_load/on_unload, wiring
├── region.rs       # Region and RegionFlags types
├── selection.rs    # SelectionStore (wand pos1/pos2 per player)
├── store.rs        # RegionStore (add/remove/list/get by position)
├── handlers.rs     # BlockBreakHandler, WandInteractHandler, WordGuardRef
└── commands/
    ├── mod.rs      # Command tree and argument names
    └── executors.rs # Define, Remove, Flag, AddOwner, AddMember, List, Wand
```

- **region** — Pure data and logic for regions/flags (no Pumpkin types in the core types).
- **selection** / **store** — State only.
- **handlers** — Event handlers and shared `WordGuardRef`.
- **commands** — Command tree and one module for all executors.

---

## Build and install

### As part of Pumpkin workspace

From the Pumpkin repo root:

```bash
cargo build -p wordguardpmc --release
cp target/release/libwordguardpmc.so plugins/
```

Then start the server; the plugin loads from `./plugins/`.

### Standalone (e.g. for worldguardalbino repo)

If this is copied or cloned as its own repo (e.g. [worldguardalbino](https://github.com/GjorgjiML/worldguardalbino)), add Pumpkin as a git or path dependency and set:

```toml
[lib]
crate-type = ["cdylib"]
```

Then build and copy the resulting `libwordguardpmc.so` into your Pumpkin server’s `plugins/` directory.

---

## Publishing to GitHub (worldguardalbino)

To publish this plugin to [github.com/GjorgjiML/worldguardalbino](https://github.com/GjorgjiML/worldguardalbino): clone that repo, copy the contents of this directory (`wordguardpmc/`) into the clone (so `Cargo.toml`, `src/`, `README.md` are at the repo root), then commit and push when ready. Adjust `Cargo.toml` dependencies to point to the Pumpkin repo (e.g. `git = "https://github.com/Pumpkin-MC/Pumpkin.git"`) if the plugin is standalone.

---

## License

Same as the Pumpkin project or as specified in the [worldguardalbino](https://github.com/GjorgjiML/worldguardalbino) repository.
