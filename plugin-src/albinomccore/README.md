# AlbionMC Core

Modular AlbionMC plugins for [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) server — **polyrepo style** with separated concerns.

## Structure

| Crate | Role |
|-------|------|
| **albion-types** | Shared models (PlayerProfile). No server or DB deps. |
| **albion-db** | PostgreSQL config, connection, migrations. Reusable. |
| **albion-core** | Pumpkin plugin — commands, events, service wiring. |

```
albinomccore/
├── albion-db/       # Database layer
├── albion-types/    # Shared types
├── albion-core/     # Pumpkin plugin (cdylib)
│   ├── src/
│   │   ├── commands/   # profile, give, admin
│   │   ├── events.rs   # PlayerJoinHandler
│   │   ├── service.rs  # ProfileService
│   │   └── state.rs    # PluginState
│   └── migrations/
└── README.md
```

## Requirements

- [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) (clone next to or include this repo)
- PostgreSQL 14+
- Rust (edition 2024)

## Setup in Pumpkin

1. Place this repo at `Pumpkin/plugin-src/albinomccore/`.

2. Add to Pumpkin's `Cargo.toml`:
```toml
[workspace]
members = [
    # ...
    "plugin-src/albinomccore/albion-db",
    "plugin-src/albinomccore/albion-types",
    "plugin-src/albinomccore/albion-core",
]
```

3. Build:
```bash
cargo build --release -p albion_core
cp target/release/libalbion_core.so plugins/
```

4. Configure `plugin-data/albion_core/config.toml`:
```toml
[database]
url = "postgres://albion:albion@localhost:5432/albionmc"
```

## Commands

| Command | Description |
|---------|-------------|
| `/albion profile` | Your profile |
| `/albion profile <player>` | Player's profile |
| `/albion give <player> silver <amount>` | Grant silver |
| `/albion give <player> fame <amount>` | Grant fame |
| `/albion admin` | Server observability |

## Database

Migrations run automatically. Schema:

```sql
CREATE TABLE albion_profiles (
    uuid UUID PRIMARY KEY,
    silver BIGINT DEFAULT 0,
    fame BIGINT DEFAULT 0,
    mastery JSONB DEFAULT '{}',
    flags JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Extending

- **New commands** → Add to `albion-core/src/commands/`.
- **New events** → Add to `albion-core/src/events.rs` or new module.
- **New types** → Add to `albion-types`.
- **New DB logic** → Add to `albion-db`.
- **New plugins** → Create new crate, depend on `albion-db` and `albion-types`.
