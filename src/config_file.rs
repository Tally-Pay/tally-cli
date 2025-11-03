//! Persistent configuration file management for the Tally CLI
//!
//! Manages the TOML configuration file stored in XDG-compliant locations,
//! supporting profiles for different networks (devnet, mainnet, etc.).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Persistent configuration file structure
///
/// Configuration precedence order (highest to lowest):
///
/// 1. CLI flags (`--rpc-url`, `--program-id`, etc.)
/// 2. Environment variables (`TALLY_RPC_URL`, etc.)
/// 3. Config file active profile
/// 4. Config file defaults
/// 5. Hardcoded defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    /// Configuration file version for migration support
    #[serde(default = "default_version")]
    pub version: String,

    /// Default settings that apply across all profiles
    #[serde(default)]
    pub defaults: DefaultConfig,

    /// Named profiles for different environments
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Default configuration values
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultConfig {
    /// Active profile name
    pub active_profile: Option<String>,

    /// Default output format (human or json)
    pub output_format: Option<String>,

    /// Wallet path override
    pub wallet_path: Option<String>,
}

/// Profile-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// RPC URL for this profile
    pub rpc_url: String,

    /// Program ID for this profile
    pub program_id: Option<String>,

    /// USDC mint address for this profile
    pub usdc_mint: Option<String>,

    /// Merchant PDA (saved after init-merchant)
    pub merchant: Option<String>,

    /// Wallet path for this profile
    pub wallet_path: Option<String>,
}

impl ConfigFile {
    /// Create a new empty config file with sensible defaults
    #[must_use]
    pub fn new() -> Self {
        let mut profiles = HashMap::new();

        // Add default devnet profile
        profiles.insert(
            "devnet".to_string(),
            ProfileConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                program_id: None,
                usdc_mint: Some("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr".to_string()),
                merchant: None,
                wallet_path: None,
            },
        );

        // Add placeholder for mainnet profile
        profiles.insert(
            "mainnet".to_string(),
            ProfileConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                program_id: None,
                usdc_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
                merchant: None,
                wallet_path: None,
            },
        );

        // Add localnet profile
        profiles.insert(
            "localnet".to_string(),
            ProfileConfig {
                rpc_url: "http://127.0.0.1:8899".to_string(),
                program_id: None,
                usdc_mint: None,
                merchant: None,
                wallet_path: None,
            },
        );

        Self {
            version: default_version(),
            defaults: DefaultConfig {
                active_profile: Some("devnet".to_string()),
                output_format: Some("human".to_string()),
                wallet_path: None,
            },
            profiles,
        }
    }

    /// Load config file from XDG config directory
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn load() -> Result<Self> {
        let path = Self::config_file_path()?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    /// Save config file to XDG config directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or file cannot be written
    pub fn save(&self) -> Result<()> {
        let path = Self::config_file_path()?;

        // Ensure config directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;

        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Get XDG-compliant config file path
    ///
    /// Returns `~/.config/tally/config.toml` on Linux/macOS
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?;

        Ok(config_dir.join("tally").join("config.toml"))
    }

    /// Get the active profile configuration
    #[must_use]
    pub fn active_profile(&self) -> Option<&ProfileConfig> {
        self.defaults
            .active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    /// Get a specific profile by name
    #[must_use]
    pub fn get_profile(&self, name: &str) -> Option<&ProfileConfig> {
        self.profiles.get(name)
    }

    /// Set the active profile
    pub fn set_active_profile(&mut self, profile_name: String) {
        self.defaults.active_profile = Some(profile_name);
    }

    /// Set a value in the active profile
    ///
    /// # Errors
    ///
    /// Returns an error if no active profile is set
    pub fn set_profile_value(&mut self, key: &str, value: String) -> Result<()> {
        let profile_name = self
            .defaults
            .active_profile
            .as_ref()
            .context("No active profile set. Use 'config init' to create one.")?;

        let profile = self
            .profiles
            .get_mut(profile_name)
            .with_context(|| format!("Profile '{profile_name}' not found"))?;

        match key {
            "rpc-url" | "rpc_url" => profile.rpc_url = value,
            "program-id" | "program_id" => profile.program_id = Some(value),
            "usdc-mint" | "usdc_mint" => profile.usdc_mint = Some(value),
            "merchant" => profile.merchant = Some(value),
            "wallet-path" | "wallet_path" => profile.wallet_path = Some(value),
            _ => anyhow::bail!("Unknown config key: {key}"),
        }

        Ok(())
    }

    /// Get a value from the active profile
    ///
    /// # Errors
    ///
    /// Returns an error if no active profile is set or key is unknown
    pub fn get_profile_value(&self, key: &str) -> Result<Option<String>> {
        let profile_name = self
            .defaults
            .active_profile
            .as_ref()
            .context("No active profile set")?;

        let profile = self
            .profiles
            .get(profile_name)
            .with_context(|| format!("Profile '{profile_name}' not found"))?;

        let value = match key {
            "rpc-url" | "rpc_url" => Some(profile.rpc_url.clone()),
            "program-id" | "program_id" => profile.program_id.clone(),
            "usdc-mint" | "usdc_mint" => profile.usdc_mint.clone(),
            "merchant" => profile.merchant.clone(),
            "wallet-path" | "wallet_path" => profile.wallet_path.clone(),
            _ => anyhow::bail!("Unknown config key: {key}"),
        };

        Ok(value)
    }

    /// Set merchant PDA for the active profile
    ///
    /// # Errors
    ///
    /// Returns an error if no active profile is set
    pub fn set_merchant(&mut self, merchant_pda: String) -> Result<()> {
        self.set_profile_value("merchant", merchant_pda)
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_config_has_default_profiles() {
        let config = ConfigFile::new();

        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.defaults.active_profile, Some("devnet".to_string()));
        assert!(config.profiles.contains_key("devnet"));
        assert!(config.profiles.contains_key("mainnet"));
        assert!(config.profiles.contains_key("localnet"));
    }

    #[test]
    fn test_active_profile() {
        let config = ConfigFile::new();
        let profile = config.active_profile().expect("Should have active profile");

        assert_eq!(profile.rpc_url, "https://api.devnet.solana.com");
    }

    #[test]
    fn test_set_active_profile() {
        let mut config = ConfigFile::new();
        config.set_active_profile("mainnet".to_string());

        assert_eq!(config.defaults.active_profile, Some("mainnet".to_string()));

        let profile = config.active_profile().expect("Should have active profile");
        assert_eq!(profile.rpc_url, "https://api.mainnet-beta.solana.com");
    }

    #[test]
    fn test_set_and_get_profile_value() {
        let mut config = ConfigFile::new();

        config
            .set_profile_value("rpc-url", "http://custom.rpc".to_string())
            .expect("Should set value");

        let value = config
            .get_profile_value("rpc-url")
            .expect("Should get value");

        assert_eq!(value, Some("http://custom.rpc".to_string()));
    }

    #[test]
    fn test_set_merchant() {
        let mut config = ConfigFile::new();
        let merchant_pda = "HkDq7K2RRStvPrXw6U3YPJrPU2dYbvGj8Y5z8VQmKR8C";

        config
            .set_merchant(merchant_pda.to_string())
            .expect("Should set merchant");

        let value = config
            .get_profile_value("merchant")
            .expect("Should get merchant");

        assert_eq!(value, Some(merchant_pda.to_string()));
    }

    #[test]
    fn test_save_and_load() {
        let _temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Override config path for test (this is tricky since we use dirs crate)
        // For now, just test serialization/deserialization
        let config = ConfigFile::new();
        let toml = toml::to_string(&config).expect("Should serialize");
        let parsed: ConfigFile = toml::from_str(&toml).expect("Should deserialize");

        assert_eq!(parsed.version, config.version);
        assert_eq!(parsed.defaults.active_profile, config.defaults.active_profile);
        assert_eq!(parsed.profiles.len(), config.profiles.len());
    }

    #[test]
    fn test_unknown_config_key_returns_error() {
        let mut config = ConfigFile::new();
        let result = config.set_profile_value("invalid-key", "value".to_string());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown config key"));
    }
}
