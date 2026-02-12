CREATE TABLE IF NOT EXISTS albion_zone_regions (
    name TEXT PRIMARY KEY,
    risk TEXT NOT NULL DEFAULT 'green',
    pvp_enabled BOOLEAN NOT NULL DEFAULT false,
    death_rule TEXT NOT NULL DEFAULT 'safe',
    partial_drop_percent SMALLINT NOT NULL DEFAULT 30,
    min_x DOUBLE PRECISION NOT NULL,
    min_y DOUBLE PRECISION NOT NULL,
    min_z DOUBLE PRECISION NOT NULL,
    max_x DOUBLE PRECISION NOT NULL,
    max_y DOUBLE PRECISION NOT NULL,
    max_z DOUBLE PRECISION NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS albion_zone_players (
    uuid UUID PRIMARY KEY,
    last_zone TEXT NOT NULL DEFAULT 'Wilderness',
    time_in_zones JSONB NOT NULL DEFAULT '{}',
    first_dangerous_entry TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_albion_zone_players_updated ON albion_zone_players(updated_at);

CREATE TABLE IF NOT EXISTS albion_zone_deaths (
    id BIGSERIAL PRIMARY KEY,
    victim_uuid UUID NOT NULL,
    killer_uuid UUID,
    zone_name TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    death_rule TEXT NOT NULL,
    items_dropped INT NOT NULL DEFAULT 0,
    items_trashed INT NOT NULL DEFAULT 0,
    died_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_albion_zone_deaths_victim ON albion_zone_deaths(victim_uuid);

CREATE INDEX IF NOT EXISTS idx_albion_zone_deaths_time ON albion_zone_deaths(died_at)
