-- AlbionMC albion_core: Player profiles v1
CREATE TABLE IF NOT EXISTS albion_profiles (
    uuid UUID PRIMARY KEY,
    silver BIGINT NOT NULL DEFAULT 0,
    fame BIGINT NOT NULL DEFAULT 0,
    mastery JSONB NOT NULL DEFAULT '{}',
    flags JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_albion_profiles_updated ON albion_profiles(updated_at);
