//! PostgreSQL connection and migrations for AlbionMC.
//!
//! Decoupled from Pumpkin — can be used by any Albion plugin.

use sqlx::PgPool;
use std::path::Path;

const DEFAULT_URL: &str = "postgres://albion:albion@localhost:5432/albionmc";

/// Load database URL from a TOML config file.
///
/// Expected structure: `[database] url = "postgres://..."`
/// If file does not exist or lacks the key, returns the default URL.
#[allow(clippy::module_name_repetitions)]
pub fn load_db_url(config_path: &Path) -> Result<String, String> {
    if !config_path.exists() {
        return Ok(DEFAULT_URL.to_string());
    }

    let config = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config: {e}"))?;
    let config: toml::Value =
        toml::from_str(&config).map_err(|e| format!("Invalid config: {e}"))?;

    let url = config
        .get("database")
        .and_then(|v| v.get("url"))
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_URL)
        .to_string();

    Ok(url)
}

/// Create default config file if it does not exist.
///
/// Returns the DB URL that was written or already present.
pub fn ensure_config(config_path: &Path, url: &str) -> Result<(), String> {
    if config_path.exists() {
        return Ok(());
    }

    let config = format!(
        r#"# AlbionMC database configuration
[database]
url = "{url}"
"#
    );
    std::fs::write(config_path, config).map_err(|e| format!("Failed to write config: {e}"))?;
    log::info!("Created default config at {:?}", config_path);
    Ok(())
}

/// Connect to PostgreSQL.
pub async fn connect(url: &str) -> Result<PgPool, String> {
    PgPool::connect(url).await.map_err(|e| format!("Failed to connect: {e}"))
}

/// Run migration SQL statements (semicolon-separated, comments stripped).
pub async fn run_migrations(pool: &PgPool, migration_sql: &str) -> Result<(), String> {
    for statement in migration_sql
        .split(';')
        .map(|s| s.split("--").next().unwrap_or(s).trim())
        .filter(|s| !s.is_empty())
    {
        sqlx::query(statement)
            .execute(pool)
            .await
            .map_err(|e| format!("Migration failed: {e}"))?;
    }
    Ok(())
}
