//! Configuration management for the Tally CLI
//!
//! Centralizes all configuration values that were previously hardcoded,
//! making them configurable via environment variables with sensible defaults.

use std::env;

/// Centralized configuration for the Tally CLI
#[derive(Debug, Clone)]
pub struct TallyCliConfig {
    /// Default RPC URL for Solana connections
    pub default_rpc_url: String,

    /// Default output format for CLI commands
    pub default_output_format: String,

    /// Default lookback time for dashboard events in seconds
    #[allow(dead_code)] // Used when dashboard functionality is re-enabled
    pub default_events_lookback_secs: i64,
}

impl TallyCliConfig {
    /// Create a new configuration instance with values from environment variables
    /// or sensible defaults if not set
    #[must_use]
    pub fn new() -> Self {
        Self {
            default_rpc_url: env::var("TALLY_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),

            default_output_format: env::var("TALLY_DEFAULT_OUTPUT_FORMAT")
                .unwrap_or_else(|_| "human".to_string()),

            default_events_lookback_secs: env::var("TALLY_DEFAULT_EVENTS_LOOKBACK_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600), // 1 hour
        }
    }

    /// Get the default lookback timestamp for dashboard events
    #[allow(dead_code)] // Used when dashboard functionality is re-enabled
    #[must_use]
    pub const fn default_events_since_timestamp(&self, current_timestamp: i64) -> i64 {
        current_timestamp - self.default_events_lookback_secs
    }
}

impl Default for TallyCliConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        // Clear any environment variables that would affect the test
        std::env::remove_var("TALLY_RPC_URL");
        std::env::remove_var("TALLY_DEFAULT_OUTPUT_FORMAT");
        std::env::remove_var("TALLY_DEFAULT_EVENTS_LOOKBACK_SECS");

        let config = TallyCliConfig::new();

        // Test that defaults are sensible
        assert_eq!(config.default_rpc_url, "https://api.devnet.solana.com");
        assert_eq!(config.default_output_format, "human");
        assert_eq!(config.default_events_lookback_secs, 3600);
    }

    #[test]
    fn test_events_timestamp() {
        let config = TallyCliConfig::new();
        let current = 7200; // 2 hours in seconds

        assert_eq!(config.default_events_since_timestamp(current), 3600); // 1 hour ago
    }
}
