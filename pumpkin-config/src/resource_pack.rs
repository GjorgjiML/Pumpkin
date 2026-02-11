use serde::{Deserialize, Serialize};

/// Configuration for server resource pack distribution.
///
/// Controls whether a resource pack is offered or enforced,
/// along with its metadata and client prompt behaviour.
/// When `force` is true, wrong pack / decline / download failure results in kick (version gating).
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ResourcePackConfig {
    /// Whether the resource pack system is enabled.
    pub enabled: bool,
    /// The URL to the resource pack (must be HTTPS and publicly reachable by clients).
    pub url: String,
    /// The SHA1 hash (40 hex characters) of the resource pack zip. Used for version gating.
    pub sha1: String,
    /// Human-readable pack version (e.g. "AlbionMC v1.3"). Shown in prompt if set; optional.
    pub pack_version: String,
    /// Custom prompt text component shown to players; leave blank for default.
    pub prompt_message: String,
    /// Whether players are forced to accept the resource pack. If true, decline/fail → kick.
    pub force: bool,
}

impl ResourcePackConfig {
    pub fn validate(&self) {
        if !self.enabled {
            return;
        }

        assert_eq!(
            !self.url.is_empty(),
            !self.sha1.is_empty(),
            "Resource pack path or SHA1 hash is missing"
        );

        let hash_len = self.sha1.len();

        assert_eq!(
            hash_len, 40,
            "Resource pack SHA1 hash is the wrong length (should be 40, is {hash_len})"
        );
    }
}
