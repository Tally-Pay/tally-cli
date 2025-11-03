//! Tally CLI - Command-line interface for the Tally subscription platform
//!
//! A comprehensive CLI tool for managing merchants, subscription plans, and subscriptions
//! on the Tally Solana-native subscription platform.

#![forbid(unsafe_code)]

mod commands;
mod config;
mod config_file;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::TallyCliConfig;
use config_file::ConfigFile;
use tally_sdk::SimpleTallyClient;

#[derive(Parser, Debug)]
#[command(
    name = "tally-merchant",
    version,
    about = "Command-line interface for the Tally subscription platform",
    author = "Tally Team"
)]
struct Cli {
    /// RPC endpoint URL
    #[arg(long, global = true)]
    rpc_url: Option<String>,

    /// Output format
    #[arg(long, value_enum, global = true)]
    output: Option<OutputFormat>,

    /// Program ID of the subscription program
    #[arg(long, global = true)]
    program_id: Option<String>,

    /// USDC mint address
    #[arg(long, global = true)]
    usdc_mint: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive setup wizard for first-time merchants
    Init {
        /// Skip the optional plan creation step
        #[arg(long)]
        skip_plan: bool,
    },

    /// Configuration commands
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Merchant account management
    Merchant {
        #[command(subcommand)]
        command: MerchantCommands,
    },

    /// Subscription plan management
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
    },

    /// Subscription management
    Subscription {
        #[command(subcommand)]
        command: SubscriptionCommands,
    },

    /// Dashboard and analytics
    Dashboard {
        #[command(subcommand)]
        command: DashboardCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Initialize a new config file with default profiles
    Init {
        /// Overwrite existing config file
        #[arg(long)]
        force: bool,
    },

    /// Show on-chain global program configuration
    Show,

    /// List all configuration values from config file
    List {
        /// Show specific profile (defaults to active profile)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Get a specific configuration value
    Get {
        /// Configuration key (e.g., rpc-url, merchant, program-id)
        key: String,

        /// Get from specific profile (defaults to active profile)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., rpc-url, merchant, program-id)
        key: String,

        /// Configuration value
        value: String,

        /// Set in specific profile (defaults to active profile)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Manage profiles
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Show config file path
    Path,
}

#[derive(Subcommand, Debug)]
enum ProfileCommands {
    /// List all available profiles
    List,

    /// Show active profile name
    Active,

    /// Set active profile
    Use {
        /// Profile name to activate
        profile: String,
    },

    /// Create a new profile
    Create {
        /// Profile name
        name: String,

        /// RPC URL for this profile
        #[arg(long)]
        rpc_url: String,

        /// Program ID (optional)
        #[arg(long)]
        program_id: Option<String>,

        /// USDC mint address (optional)
        #[arg(long)]
        usdc_mint: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum MerchantCommands {
    /// Initialize a new merchant account
    Init {
        /// Authority keypair for the merchant
        #[arg(long)]
        authority: Option<String>,

        /// USDC treasury account for the merchant
        #[arg(long)]
        treasury: String,

        /// Fee basis points (e.g., 50 = 0.5%)
        #[arg(long)]
        fee_bps: u16,
    },

    /// Show merchant account details
    Show {
        /// Merchant account address
        #[arg(long)]
        merchant: String,
    },
}

#[derive(Subcommand, Debug)]
enum PlanCommands {
    /// Create a new subscription plan
    Create {
        /// Merchant account address
        #[arg(long)]
        merchant: String,

        /// Plan identifier
        #[arg(long)]
        id: String,

        /// Plan display name
        #[arg(long)]
        name: String,

        /// Price in USDC (e.g., 10.0 for $10 USDC)
        #[arg(long = "price-usdc")]
        price_usdc: f64,

        /// Billing period in days (e.g., 30 for monthly)
        #[arg(long = "period-days", conflicts_with = "period_months")]
        period_days: Option<u32>,

        /// Billing period in months (convenient shortcut)
        #[arg(long = "period-months", conflicts_with = "period_days")]
        period_months: Option<u32>,

        /// Grace period in days (defaults to 1 day if not specified)
        #[arg(long = "grace-days", default_value = "1")]
        grace_days: u32,

        /// Authority keypair for the merchant
        #[arg(long)]
        authority: Option<String>,
    },

    /// List all plans for a merchant
    List {
        /// Merchant account address
        #[arg(long)]
        merchant: String,
    },

    /// Update subscription plan terms (price, period, grace period)
    Update {
        /// Plan account address
        #[arg(long)]
        plan: String,

        /// New price in USDC (e.g., 15.0 for $15 USDC)
        #[arg(long = "price-usdc")]
        price_usdc: Option<f64>,

        /// New billing period in days
        #[arg(long = "period-days", conflicts_with = "period_months")]
        period_days: Option<u32>,

        /// New billing period in months (convenient shortcut)
        #[arg(long = "period-months", conflicts_with = "period_days")]
        period_months: Option<u32>,

        /// New grace period in days
        #[arg(long = "grace-days")]
        grace_days: Option<u32>,

        /// Authority keypair for the merchant
        #[arg(long)]
        authority: Option<String>,
    },

    /// Deactivate a subscription plan
    Deactivate {
        /// Plan account address
        #[arg(long)]
        plan: String,

        /// Authority keypair for the merchant
        #[arg(long)]
        authority: Option<String>,

        /// Skip confirmation prompt (for scripts)
        #[arg(long, short = 'y')]
        yes: bool,

        /// Preview the operation without executing it
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SubscriptionCommands {
    /// List subscriptions for a plan
    List {
        /// Plan account address
        #[arg(long)]
        plan: String,
    },

    /// Show subscription account details
    Show {
        /// Subscription account address
        #[arg(long)]
        subscription: String,
    },
}

#[derive(Subcommand, Debug)]
enum DashboardCommands {
    /// Display merchant overview statistics
    Overview {
        /// Merchant account address
        #[arg(long)]
        merchant: String,
    },

    /// Show analytics for a specific plan
    Analytics {
        /// Plan account address
        #[arg(long)]
        plan: String,
    },

    /// Monitor real-time events for a merchant
    Events {
        /// Merchant account address
        #[arg(long)]
        merchant: String,

        /// Only show events since this timestamp
        #[arg(long)]
        since: Option<i64>,
    },

    /// List subscriptions for a merchant with enhanced information
    Subscriptions {
        /// Merchant account address
        #[arg(long)]
        merchant: String,

        /// Only show active subscriptions
        #[arg(long)]
        active_only: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config = TallyCliConfig::new();

    // Load config file (if it exists) for additional defaults
    let config_file = ConfigFile::load().unwrap_or_else(|_| ConfigFile::new());

    let default_output_format = parse_output_format(&config.default_output_format)?;
    let output_format = cli.output.as_ref().unwrap_or(&default_output_format);

    // Check if this command needs SDK access (on-chain operations)
    let needs_sdk = command_needs_sdk(&cli.command);

    // Only initialize SDK client if the command requires on-chain access
    let result = if needs_sdk {
        // Use configuration with precedence: CLI flags > env vars > config file > defaults
        let rpc_url = cli
            .rpc_url
            .as_deref()
            .or_else(|| std::env::var("TALLY_RPC_URL").ok().as_deref().map(|_| config.default_rpc_url.as_str()))
            .or_else(|| config_file.active_profile().map(|p| p.rpc_url.as_str()))
            .unwrap_or(&config.default_rpc_url);

        // Program ID precedence
        let program_id_from_config = cli
            .program_id
            .as_deref()
            .or_else(|| {
                config_file
                    .active_profile()
                    .and_then(|p| p.program_id.as_deref())
            });

        // Check if program ID is available before trying to create client
        let program_id = if let Some(id) = program_id_from_config {
            id
        } else if std::env::var("TALLY_PROGRAM_ID").is_err() {
            // Neither config nor env var has program ID
            return Err(anyhow::anyhow!(
                "This command requires connection to Solana.\n\
                 \n\
                 Program ID not configured. You can fix this by:\n\
                 \n\
                 1. Set the TALLY_PROGRAM_ID environment variable:\n\
                    export TALLY_PROGRAM_ID=<your-program-id>\n\
                 \n\
                 2. Or configure it in your profile:\n\
                    tally-merchant config init\n\
                    tally-merchant config set program-id <your-program-id>\n\
                 \n\
                 3. Or pass it as a CLI flag:\n\
                    tally-merchant --program-id <your-program-id> <command>\n\
                 \n\
                 See https://github.com/Tally-Pay/tally-cli for program IDs"
            ));
        } else {
            // If env var is set, use empty string to let SDK read it
            ""
        };

        // Initialize Tally client
        let tally_client = if program_id.is_empty() {
            SimpleTallyClient::new(rpc_url)?
        } else {
            SimpleTallyClient::new_with_program_id(rpc_url, program_id)?
        };

        // Execute command with SDK client
        execute_command(&cli, Some(&tally_client), &config).await
    } else {
        // Execute command without SDK client (config file operations)
        execute_command(&cli, None, &config).await
    };

    // Handle output formatting
    match result {
        Ok(output) => match output_format {
            OutputFormat::Human => println!("{output}"),
            OutputFormat::Json => {
                let json_output = serde_json::json!({
                    "success": true,
                    "data": output
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            }
        },
        Err(e) => {
            match output_format {
                OutputFormat::Human => eprintln!("Error: {e}"),
                OutputFormat::Json => {
                    let json_output = serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    });
                    println!("{}", serde_json::to_string_pretty(&json_output)?);
                }
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Check if a command requires SDK access (on-chain operations)
#[allow(clippy::missing_const_for_fn)]
fn command_needs_sdk(command: &Commands) -> bool {
    match command {
        Commands::Config { command } => matches!(command, ConfigCommands::Show),
        Commands::Init { .. }
        | Commands::Merchant { .. }
        | Commands::Plan { .. }
        | Commands::Subscription { .. }
        | Commands::Dashboard { .. } => true,
    }
}

/// Parse output format from string
fn parse_output_format(format_str: &str) -> Result<OutputFormat> {
    match format_str.to_lowercase().as_str() {
        "human" => Ok(OutputFormat::Human),
        "json" => Ok(OutputFormat::Json),
        _ => Err(anyhow::anyhow!("Invalid output format: {format_str}")),
    }
}

/// Convert USDC decimal to micro-units with validation
fn usdc_to_micro_units(usdc: f64) -> Result<u64> {
    if usdc < 0.0 {
        return Err(anyhow::anyhow!("Price must be greater than or equal to 0"));
    }
    if usdc > 1_000_000.0 {
        return Err(anyhow::anyhow!(
            "Price seems too high: ${usdc}. Did you mean ${:.2}?",
            usdc / 1_000_000.0
        ));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let micro_units = (usdc * 1_000_000.0) as u64;
    Ok(micro_units)
}

/// Execute config commands
async fn execute_config_commands(
    cli: &Cli,
    tally_client: Option<&SimpleTallyClient>,
    config: &TallyCliConfig,
    command: &ConfigCommands,
) -> Result<String> {
    match command {
        ConfigCommands::Init { force } => commands::config_file_ops::init(*force),

        ConfigCommands::Show => {
            let client = require_client(tally_client)?;
            let output_format = match cli.output {
                Some(OutputFormat::Json) => "json",
                _ => "human",
            };
            let request = commands::show_config::ShowConfigRequest { output_format };
            commands::execute_show_config(client, &request, config).await
        }

        ConfigCommands::List { profile } => {
            commands::config_file_ops::list(profile.as_deref())
        }

        ConfigCommands::Get { key, profile } => {
            commands::config_file_ops::get(key, profile.as_deref())
        }

        ConfigCommands::Set {
            key,
            value,
            profile,
        } => commands::config_file_ops::set(key, value, profile.as_deref()),

        ConfigCommands::Path => commands::config_file_ops::path(),

        ConfigCommands::Profile { command } => match command {
            ProfileCommands::List => commands::config_file_ops::list_profiles(),
            ProfileCommands::Active => commands::config_file_ops::show_active_profile(),
            ProfileCommands::Use { profile } => {
                commands::config_file_ops::use_profile(profile)
            }
            ProfileCommands::Create {
                name,
                rpc_url,
                program_id,
                usdc_mint,
            } => commands::config_file_ops::create_profile(
                name,
                rpc_url,
                program_id.as_deref(),
                usdc_mint.as_deref(),
            ),
        },
    }
}

/// Execute merchant commands
async fn execute_merchant_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    command: &MerchantCommands,
) -> Result<String> {
    match command {
        MerchantCommands::Init {
            authority,
            treasury,
            fee_bps,
        } => {
            commands::execute_init_merchant(
                tally_client,
                authority.as_deref(),
                treasury,
                *fee_bps,
                cli.usdc_mint.as_deref(),
                config,
            )
            .await
        }

        MerchantCommands::Show { merchant } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => "json",
                _ => "human",
            };
            let request = commands::show_merchant::ShowMerchantRequest {
                merchant,
                output_format,
            };
            commands::execute_show_merchant(tally_client, &request, config).await
        }
    }
}

/// Execute plan commands
async fn execute_plan_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    command: &PlanCommands,
) -> Result<String> {
    match command {
        PlanCommands::Create {
            merchant,
            id,
            name,
            price_usdc,
            period_days,
            period_months,
            grace_days,
            authority,
        } => {
            // Convert USDC to micro-units (6 decimals) with validation
            let price_micro = usdc_to_micro_units(*price_usdc)?;

            // Convert period to seconds (prefer days, allow months as alternative)
            let period_secs = period_months.map_or_else(
                || {
                    period_days.map_or_else(
                        || {
                            Err(anyhow::anyhow!(
                                "Either --period-days or --period-months is required"
                            ))
                        },
                        |days| Ok(i64::from(days) * 86400),
                    )
                },
                |months| Ok(i64::from(months) * 30 * 86400),
            )?;

            // Convert grace period to seconds
            let grace_secs = i64::from(*grace_days) * 86400;

            let request = commands::create_plan::CreatePlanRequest {
                merchant_str: merchant,
                plan_id: id,
                plan_name: name,
                price_usdc: price_micro,
                period_secs,
                grace_secs,
                authority_path: authority.as_deref(),
            };
            commands::execute_create_plan(tally_client, &request, config).await
        }

        PlanCommands::List { merchant } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => commands::list_plans::OutputFormat::Json,
                _ => commands::list_plans::OutputFormat::Human,
            };
            commands::execute_list_plans(tally_client, merchant, &output_format).await
        }

        PlanCommands::Update {
            plan,
            price_usdc,
            period_days,
            period_months,
            grace_days,
            authority,
        } => {
            // Convert USDC to micro-units if provided
            let new_price = if let Some(p) = price_usdc {
                Some(usdc_to_micro_units(*p)?)
            } else {
                None
            };

            // Convert period to seconds if provided (prefer days, allow months)
            let new_period_seconds = period_months.map_or_else(
                || period_days.map(|d| i64::from(d) * 86400),
                |months| Some(i64::from(months) * 30 * 86400),
            );

            // Convert grace period to seconds if provided
            let new_grace_period_seconds = grace_days.map(|d| i64::from(d) * 86400);

            let request = commands::update_plan_terms::UpdatePlanTermsRequest {
                plan,
                new_price,
                new_period_seconds,
                new_grace_period_seconds,
            };
            commands::execute_update_plan_terms(
                tally_client,
                &request,
                authority.as_deref(),
                config,
            )
            .await
        }

        PlanCommands::Deactivate {
            plan,
            authority,
            yes,
            dry_run,
        } => {
            commands::execute_deactivate_plan(
                tally_client,
                plan,
                authority.as_deref(),
                *yes,
                *dry_run,
            )
            .await
        }
    }
}

/// Execute subscription commands
async fn execute_subscription_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    command: &SubscriptionCommands,
) -> Result<String> {
    match command {
        SubscriptionCommands::List { plan } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => commands::list_subs::OutputFormat::Json,
                _ => commands::list_subs::OutputFormat::Human,
            };
            commands::execute_list_subs(tally_client, plan, &output_format, config).await
        }

        SubscriptionCommands::Show { subscription } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => "json",
                _ => "human",
            };
            let request = commands::show_subscription::ShowSubscriptionRequest {
                subscription,
                output_format,
            };
            commands::execute_show_subscription(tally_client, &request, config).await
        }
    }
}

/// Execute dashboard commands
fn execute_dashboard_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    command: &DashboardCommands,
) -> Result<String> {
    let output_format = match cli.output {
        Some(OutputFormat::Json) => commands::dashboard::OutputFormat::Json,
        _ => commands::dashboard::OutputFormat::Human,
    };
    let rpc_url = cli.rpc_url.as_deref().unwrap_or(&config.default_rpc_url);
    commands::dashboard::execute(tally_client, command, &output_format, rpc_url, config)
}

/// Main command router
async fn execute_command(
    cli: &Cli,
    tally_client: Option<&SimpleTallyClient>,
    config: &TallyCliConfig,
) -> Result<String> {
    match &cli.command {
        Commands::Init { skip_plan } => {
            let client = require_client(tally_client)?;
            commands::execute_init_wizard(client, config, *skip_plan).await
        }
        Commands::Config { command } => {
            execute_config_commands(cli, tally_client, config, command).await
        }
        Commands::Merchant { command } => {
            let client = require_client(tally_client)?;
            execute_merchant_commands(cli, client, config, command).await
        }
        Commands::Plan { command } => {
            let client = require_client(tally_client)?;
            execute_plan_commands(cli, client, config, command).await
        }
        Commands::Subscription { command } => {
            let client = require_client(tally_client)?;
            execute_subscription_commands(cli, client, config, command).await
        }
        Commands::Dashboard { command } => {
            let client = require_client(tally_client)?;
            execute_dashboard_commands(cli, client, config, command)
        }
    }
}

/// Helper to require SDK client with helpful error message
fn require_client(client: Option<&SimpleTallyClient>) -> Result<&SimpleTallyClient> {
    client.ok_or_else(|| {
        anyhow::anyhow!(
            "This command requires connection to Solana.\n\
             \n\
             Program ID not configured. You can fix this by:\n\
             \n\
             1. Set the TALLY_PROGRAM_ID environment variable:\n\
                export TALLY_PROGRAM_ID=<your-program-id>\n\
             \n\
             2. Or configure it in your profile:\n\
                tally-merchant config init\n\
                tally-merchant config set program-id <your-program-id>\n\
             \n\
             3. Or pass it as a CLI flag:\n\
                tally-merchant --program-id <your-program-id> <command>\n\
             \n\
             See https://github.com/Tally-Pay/tally-cli for program IDs"
        )
    })
}
