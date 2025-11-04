//! Config file operations command handlers

use crate::config_file::{ConfigFile, ProfileConfig};
use crate::utils::colors::Theme;
use anyhow::{Context, Result};
use std::fmt::Write;

/// Initialize a new config file
///
/// # Errors
///
/// Returns an error if the config file already exists (unless `force` is true),
/// or if the file cannot be created or written
pub fn init(force: bool) -> Result<String> {
    let path = ConfigFile::config_file_path()?;

    // Check if config file already exists
    if path.exists() && !force {
        return Err(anyhow::anyhow!(
            "Config file already exists at: {}\n\
             Use --force to overwrite",
            path.display()
        ));
    }

    // Create new config with default profiles
    let config = ConfigFile::new();
    config.save()?;

    let mut output = String::new();
    writeln!(&mut output, "{} Config file initialized at: {}", Theme::success("✓"), Theme::value(&path.display().to_string()))?;
    writeln!(&mut output)?;
    writeln!(&mut output, "{}", Theme::header("Default profiles created:"))?;
    writeln!(&mut output, "  {} {}", Theme::dim("•"), Theme::active("devnet (active)"))?;
    writeln!(&mut output, "  {} mainnet", Theme::dim("•"))?;
    writeln!(&mut output, "  {} localnet", Theme::dim("•"))?;
    writeln!(&mut output)?;
    writeln!(&mut output, "{}", Theme::dim("Use 'tally-merchant config list' to view configuration"))?;
    write!(&mut output, "{}", Theme::dim("Use 'tally-merchant config set <key> <value>' to customize"))?;
    Ok(output)
}

/// List all configuration values
///
/// # Errors
///
/// Returns an error if the config file cannot be loaded or if the specified profile is not found
pub fn list(profile_name: Option<&str>) -> Result<String> {
    let config = ConfigFile::load()?;

    let profile_to_show = if let Some(name) = &profile_name {
        config
            .get_profile(name)
            .with_context(|| format!("Profile '{name}' not found"))?
    } else {
        config
            .active_profile()
            .context("No active profile set. Run 'tally-merchant config init'")?
    };

    let profile_name_display = profile_name
        .map(String::from)
        .or_else(|| config.defaults.active_profile.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let mut output = String::new();
    writeln!(&mut output, "{} (profile: {})", Theme::header("Configuration"), Theme::highlight(&profile_name_display))?;
    writeln!(&mut output, "{}", Theme::dim(&"=".repeat(50)))?;

    writeln!(&mut output, "{:<15} {}", Theme::info("RPC URL:"), Theme::value(&profile_to_show.rpc_url))?;
    writeln!(&mut output, "{:<15} {}",
        Theme::info("Program ID:"),
        profile_to_show.program_id.as_ref().map_or_else(
            || Theme::dim("(not set)"),
            |v| Theme::value(v)
        ))?;
    writeln!(&mut output, "{:<15} {}",
        Theme::info("USDC Mint:"),
        profile_to_show.usdc_mint.as_ref().map_or_else(
            || Theme::dim("(not set)"),
            |v| Theme::dim(v)
        ))?;
    writeln!(&mut output, "{:<15} {}",
        Theme::info("Merchant:"),
        profile_to_show.merchant.as_ref().map_or_else(
            || Theme::dim("(not set)"),
            |v| Theme::highlight(v)
        ))?;
    write!(&mut output, "{:<15} {}",
        Theme::info("Wallet Path:"),
        profile_to_show.wallet_path.as_ref().map_or_else(
            || Theme::dim("(default)"),
            |v| Theme::value(v)
        ))?;

    Ok(output)
}

/// Get a specific configuration value
///
/// # Errors
///
/// Returns an error if the config file cannot be loaded, the profile is not found, or the key is invalid
pub fn get(key: &str, profile_name: Option<&str>) -> Result<String> {
    let mut config = ConfigFile::load()?;

    // Temporarily set active profile if specified
    let original_active = config.defaults.active_profile.clone();
    if let Some(name) = profile_name {
        config.set_active_profile(name.to_string());
    }

    let value = config.get_profile_value(key)?;

    // Restore original active profile
    if let Some(original) = original_active {
        config.set_active_profile(original);
    }

    value.map_or_else(|| Ok("(not set)".to_string()), Ok)
}

/// Set a configuration value
///
/// # Errors
///
/// Returns an error if the config file cannot be loaded or saved, the profile is not found, or the key is invalid
pub fn set(key: &str, value: &str, profile_name: Option<&str>) -> Result<String> {
    let mut config = ConfigFile::load()?;

    // Temporarily set active profile if specified
    let original_active = config.defaults.active_profile.clone();
    if let Some(name) = profile_name {
        config.set_active_profile(name.to_string());
    }

    config.set_profile_value(key, value.to_string())?;

    // Restore original active profile before saving
    if let Some(original) = original_active {
        config.set_active_profile(original);
    }

    config.save()?;

    let profile_display = profile_name
        .map(String::from)
        .or(config.defaults.active_profile)
        .unwrap_or_else(|| "unknown".to_string());

    Ok(format!(
        "{} Set {} = {} (profile: {})",
        Theme::success("✓"),
        Theme::info(key),
        Theme::value(value),
        Theme::highlight(&profile_display)
    ))
}

/// Show config file path
///
/// # Errors
///
/// Returns an error if the config directory cannot be determined
pub fn path() -> Result<String> {
    let path = ConfigFile::config_file_path()?;
    Ok(format!("{}", path.display()))
}

/// List all available profiles
///
/// # Errors
///
/// Returns an error if the config file cannot be loaded
pub fn list_profiles() -> Result<String> {
    let config = ConfigFile::load()?;

    let mut output = String::new();
    writeln!(&mut output, "{}", Theme::header("Available Profiles:"))?;
    writeln!(&mut output, "{}", Theme::dim(&"=".repeat(50)))?;

    let active_profile = config.defaults.active_profile.as_deref();

    for (name, profile) in &config.profiles {
        let is_active = active_profile == Some(name.as_str());
        let name_display = if is_active {
            format!("{} {}", Theme::highlight(name), Theme::active("(active)"))
        } else {
            Theme::value(name).to_string()
        };

        writeln!(&mut output)?;
        writeln!(&mut output, "{name_display}")?;
        writeln!(&mut output, "  {}: {}", Theme::dim("RPC URL"), profile.rpc_url)?;
        if let Some(ref program_id) = profile.program_id {
            writeln!(&mut output, "  {}: {}", Theme::dim("Program ID"), Theme::dim(program_id))?;
        }
        if let Some(ref merchant) = profile.merchant {
            writeln!(&mut output, "  {}: {}", Theme::dim("Merchant"), Theme::highlight(merchant))?;
        }
    }

    Ok(output)
}

/// Show active profile name
///
/// # Errors
///
/// Returns an error if the config file cannot be loaded
pub fn show_active_profile() -> Result<String> {
    let config = ConfigFile::load()?;

    Ok(config.defaults.active_profile.unwrap_or_else(|| "(none)".to_string()))
}

/// Set active profile
///
/// # Errors
///
/// Returns an error if the config file cannot be loaded or saved, or if the specified profile does not exist
pub fn use_profile(profile_name: &str) -> Result<String> {
    let mut config = ConfigFile::load()?;

    // Verify profile exists
    if !config.profiles.contains_key(profile_name) {
        return Err(anyhow::anyhow!(
            "Profile '{profile_name}' not found.\n\
             \n\
             Available profiles:\n  {}\n\
             \n\
             Use 'tally-merchant config profile create' to create a new profile",
            config
                .profiles
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join("\n  ")
        ));
    }

    config.set_active_profile(profile_name.to_string());
    config.save()?;

    Ok(format!("{} Active profile set to: {}", Theme::success("✓"), Theme::highlight(profile_name)))
}

/// Create a new profile
///
/// # Errors
///
/// Returns an error if the config file cannot be loaded or saved, or if a profile with the same name already exists
pub fn create_profile(
    name: &str,
    rpc_url: &str,
    program_id: Option<&str>,
    usdc_mint: Option<&str>,
) -> Result<String> {
    let mut config = ConfigFile::load()?;

    // Check if profile already exists
    if config.profiles.contains_key(name) {
        return Err(anyhow::anyhow!(
            "Profile '{name}' already exists.\n\
             Use 'tally-merchant config set' to modify existing profiles"
        ));
    }

    // Create new profile
    let profile = ProfileConfig {
        rpc_url: rpc_url.to_string(),
        program_id: program_id.map(String::from),
        usdc_mint: usdc_mint.map(String::from),
        merchant: None,
        wallet_path: None,
    };

    config.profiles.insert(name.to_string(), profile);
    config.save()?;

    let mut output = String::new();
    writeln!(&mut output, "{} Profile '{}' created with:", Theme::success("✓"), Theme::highlight(name))?;
    writeln!(&mut output, "  {} RPC URL: {}", Theme::dim("•"), Theme::value(rpc_url))?;
    writeln!(&mut output)?;
    write!(&mut output, "{}", Theme::dim(&format!("Use 'tally-merchant config profile use {name}' to activate")))?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_returns_valid_path() {
        let result = path();
        assert!(result.is_ok());
        assert!(result.unwrap().contains("config.toml"));
    }
}
