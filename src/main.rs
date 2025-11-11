//! Tally CLI - Command-line interface for the Tally subscription platform
//!
//! A comprehensive CLI tool for managing merchants, subscription plans, and subscriptions
//! on the Tally Solana-native subscription platform.

#![forbid(unsafe_code)]

mod commands;
mod config;
mod config_file;
mod errors;
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
pub struct Cli {
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

    /// Disable color output (respects `NO_COLOR` environment variable)
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Csv,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive setup wizard for first-time merchants
    #[command(
        long_about = "Launch an interactive wizard to set up your merchant account.\n\n\
                             This guided setup will:\n\
                             • Check your wallet and SOL balance\n\
                             • Help you configure a USDC treasury\n\
                             • Set your merchant fee\n\
                             • Create your merchant account\n\
                             • Optionally guide you to create your first plan\n\n\
                             Example:\n  \
                             tally-merchant init"
    )]
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

    /// Payee account management
    Payee {
        #[command(subcommand)]
        command: PayeeCommands,
    },

    /// Payment terms management
    PaymentTerms {
        #[command(subcommand)]
        command: PaymentTermsCommands,
    },

    /// Payment agreement management
    Agreement {
        #[command(subcommand)]
        command: AgreementCommands,
    },

    /// Dashboard and analytics
    Dashboard {
        #[command(subcommand)]
        command: DashboardCommands,
    },

    /// Generate and install shell completions
    #[command(
        long_about = "Generate and install shell completion scripts for your shell.\n\n\
                             By default, this command will guide you through interactive installation.\n\
                             Use --print to output the completion script for manual installation.\n\n\
                             Examples:\n  \
                             # Interactive installation (recommended)\n  \
                             tally-merchant completions zsh\n\n  \
                             # Automatic installation (skip prompts)\n  \
                             tally-merchant completions zsh --install --yes\n\n  \
                             # Preview installation plan\n  \
                             tally-merchant completions zsh --dry-run\n\n  \
                             # Print script for manual installation\n  \
                             tally-merchant completions zsh --print > ~/.zsh/completions/_tally-merchant\n\n  \
                             # Uninstall completions\n  \
                             tally-merchant completions zsh --uninstall"
    )]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,

        /// Install completions automatically (interactive)
        #[arg(long)]
        install: bool,

        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,

        /// Print completion script to stdout (for manual installation)
        #[arg(long)]
        print: bool,

        /// Show what would be installed without making changes
        #[arg(long)]
        dry_run: bool,

        /// Remove installed completions
        #[arg(long)]
        uninstall: bool,
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

    /// Show specific profile configuration
    Show {
        /// Profile name to display (defaults to active profile)
        profile: Option<String>,
    },

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
enum PayeeCommands {
    /// Initialize a new merchant account
    #[command(
        long_about = "Initialize a new merchant account on the Tally protocol.\n\n\
                             This creates your merchant PDA (Program Derived Address) which will\n\
                             be used to manage subscription plans. New merchants are automatically\n\
                             assigned to the Free tier with a 2.0% platform fee.\n\n\
                             Prerequisites:\n  \
                             • SOL for transaction fees (~0.01 SOL)\n  \
                             • USDC treasury ATA (will be auto-created if it doesn't exist)\n\n\
                             Arguments:\n  \
                             --treasury: USDC Associated Token Account for receiving payments\n\n\
                             Examples:\n  \
                             # Initialize merchant account\n  \
                             tally-merchant merchant init \\\n    \
                             --treasury <USDC_ATA>\n\n\
                             Note: For first-time setup, use 'tally-merchant init' instead,\n\
                             which provides an interactive wizard.\n\n\
                             Your merchant starts on the Free tier (2.0% platform fee).\n\
                             Contact the platform authority to upgrade to Pro (1.5%) or Enterprise (1.0%) tiers."
    )]
    Init {
        /// Authority keypair for the merchant
        #[arg(long)]
        authority: Option<String>,

        /// USDC treasury account for the merchant (ATA will be created if needed)
        #[arg(
            long,
            help = "USDC Associated Token Account for receiving subscription payments"
        )]
        treasury: String,
    },

    /// Show payee account details
    Show {
        /// Payee account address
        #[arg(long)]
        payee: String,
    },
}

#[derive(Subcommand, Debug)]
enum PaymentTermsCommands {
    /// Create a new subscription plan
    #[command(long_about = "Create a new subscription plan for your merchant.\n\n\
                             A subscription plan defines the price, billing period, and grace period\n\
                             for recurring USDC payments. Once created, users can subscribe to the plan\n\
                             via Solana Actions (Blinks).\n\n\
                             Arguments:\n  \
                             --price-usdc: Price in USDC (e.g., 10.0 for $10/month)\n  \
                             --period-days: Billing period in days (e.g., 30 for monthly)\n  \
                             --period-months: Alternative to period-days (e.g., 1 for monthly)\n  \
                             --grace-days: Days after missed payment before cancellation (default: 1)\n\n\
                             Examples:\n  \
                             # Create a $10/month premium plan\n  \
                             tally-merchant plan create \\\n    \
                             --merchant <MERCHANT_PDA> \\\n    \
                             --id premium \\\n    \
                             --name \"Premium Plan\" \\\n    \
                             --price-usdc 10.0 \\\n    \
                             --period-months 1\n\n  \
                             # Create a $50/quarter business plan with 3-day grace\n  \
                             tally-merchant plan create \\\n    \
                             --merchant <MERCHANT_PDA> \\\n    \
                             --id business \\\n    \
                             --name \"Business Plan\" \\\n    \
                             --price-usdc 50.0 \\\n    \
                             --period-days 90 \\\n    \
                             --grace-days 3")]
    Create {
        /// Payee account address
        #[arg(long)]
        payee: String,

        /// Payment terms identifier (used in PDA)
        #[arg(
            long,
            help = "Unique identifier for these payment terms (e.g., 'premium', 'basic')"
        )]
        id: String,

        /// Amount in USDC (e.g., 10.0 for $10 USDC)
        #[arg(
            long = "amount-usdc",
            help = "Payment amount in USDC (e.g., 10.0 for $10/period)"
        )]
        amount_usdc: f64,

        /// Billing period in days (e.g., 30 for monthly)
        #[arg(
            long = "period-days",
            conflicts_with = "period_months",
            help = "Billing period in days (e.g., 30 for monthly, 365 for yearly)"
        )]
        period_days: Option<u32>,

        /// Billing period in months (convenient shortcut)
        #[arg(
            long = "period-months",
            conflicts_with = "period_days",
            help = "Billing period in months (e.g., 1 for monthly, 12 for yearly)"
        )]
        period_months: Option<u32>,

        /// Authority keypair for the payee
        #[arg(long)]
        authority: Option<String>,
    },

    /// List all payment terms for a payee
    List {
        /// Payee account address
        #[arg(long)]
        payee: String,
    },
}

#[derive(Subcommand, Debug)]
enum AgreementCommands {
    /// List payment agreements for payment terms
    List {
        /// Payment terms account address
        #[arg(long)]
        payment_terms: String,
    },

    /// Show payment agreement account details
    Show {
        /// Payment agreement account address
        #[arg(long)]
        agreement: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum DashboardCommands {
    /// Display merchant overview statistics
    Overview {
        /// Merchant account address (defaults to merchant from active profile)
        #[arg(long)]
        merchant: Option<String>,
    },

    /// Show analytics for a specific plan
    Analytics {
        /// Plan account address
        #[arg(long)]
        plan: String,
    },

    /// Monitor real-time events for a merchant
    Events {
        /// Merchant account address (defaults to merchant from active profile)
        #[arg(long)]
        merchant: Option<String>,

        /// Only show events since this timestamp
        #[arg(long)]
        since: Option<i64>,
    },

    /// List subscriptions for a merchant with enhanced information
    Subscriptions {
        /// Merchant account address (defaults to merchant from active profile)
        #[arg(long)]
        merchant: Option<String>,

        /// Only show active subscriptions
        #[arg(long)]
        active_only: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Initialize colors based on --no-color flag and NO_COLOR env var
    utils::colors::init_colors(cli.no_color);

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
            .or_else(|| {
                std::env::var("TALLY_RPC_URL")
                    .ok()
                    .as_deref()
                    .map(|_| config.default_rpc_url.as_str())
            })
            .or_else(|| config_file.active_profile().map(|p| p.rpc_url.as_str()))
            .unwrap_or(&config.default_rpc_url);

        // Program ID precedence
        let program_id_from_config = cli.program_id.as_deref().or_else(|| {
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
        execute_command(&cli, Some(&tally_client), &config, &config_file).await
    } else {
        // Execute command without SDK client (config file operations)
        execute_command(&cli, None, &config, &config_file).await
    };

    // Handle output formatting
    match result {
        Ok(output) => match output_format {
            OutputFormat::Human => {
                // Output is already formatted by commands, just print it
                println!("{output}");
            }
            OutputFormat::Json => {
                let json_output = serde_json::json!({
                    "success": true,
                    "data": output
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            }
            OutputFormat::Csv => {
                // CSV is already formatted by commands, just print it
                println!("{output}");
            }
        },
        Err(e) => {
            match output_format {
                OutputFormat::Human => {
                    eprintln!("{}: {e}", utils::colors::Theme::error("Error"));
                }
                OutputFormat::Json => {
                    let json_output = serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    });
                    println!("{}", serde_json::to_string_pretty(&json_output)?);
                }
                OutputFormat::Csv => {
                    // For CSV, output error as plain text to stderr
                    eprintln!("Error: {e}");
                }
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Check if a command requires SDK access (on-chain operations)
const fn command_needs_sdk(command: &Commands) -> bool {
    match command {
        Commands::Config { command } => matches!(command, ConfigCommands::Show),
        Commands::Init { .. }
        | Commands::Payee { .. }
        | Commands::PaymentTerms { .. }
        | Commands::Agreement { .. }
        | Commands::Dashboard { .. } => true,
        Commands::Completions { .. } => false,
    }
}

/// Parse output format from string
fn parse_output_format(format_str: &str) -> Result<OutputFormat> {
    match format_str.to_lowercase().as_str() {
        "human" => Ok(OutputFormat::Human),
        "json" => Ok(OutputFormat::Json),
        "csv" => Ok(OutputFormat::Csv),
        _ => Err(anyhow::anyhow!("Invalid output format: {format_str}")),
    }
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

        ConfigCommands::List { profile } => commands::config_file_ops::list(profile.as_deref()),

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
            ProfileCommands::Show { profile } => {
                commands::config_file_ops::show_profile(profile.as_deref())
            }
            ProfileCommands::Use { profile } => commands::config_file_ops::use_profile(profile),
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

/// Execute payee commands
async fn execute_payee_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    command: &PayeeCommands,
) -> Result<String> {
    match command {
        PayeeCommands::Init {
            authority,
            treasury,
        } => {
            commands::execute_init_payee(
                tally_client,
                authority.as_deref(),
                treasury,
                cli.usdc_mint.as_deref(),
                config,
            )
            .await
        }

        PayeeCommands::Show { payee } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => "json",
                _ => "human",
            };
            let request = commands::show_payee::ShowPayeeRequest {
                payee,
                output_format,
            };
            commands::execute_show_payee(tally_client, &request, config).await
        }
    }
}

/// Execute payment terms commands
async fn execute_payment_terms_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    command: &PaymentTermsCommands,
) -> Result<String> {
    match command {
        PaymentTermsCommands::Create {
            payee,
            id,
            amount_usdc,
            period_days,
            period_months,
            authority,
        } => {
            // Convert period to days (prefer days, allow months as alternative)
            let days = period_months.map_or_else(
                || {
                    period_days.map_or_else(
                        || {
                            Err(anyhow::anyhow!(
                                "Either --period-days or --period-months is required"
                            ))
                        },
                        |days| Ok(u64::from(days)),
                    )
                },
                |months| Ok(u64::from(months) * 30),
            )?;

            let request = commands::create_payment_terms::CreatePaymentTermsRequest {
                payee_str: payee,
                terms_id: id,
                amount_usdc_float: *amount_usdc,
                period_days: days,
                authority_path: authority.as_deref(),
            };
            commands::execute_create_payment_terms(tally_client, &request, config).await
        }

        PaymentTermsCommands::List { payee } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => commands::list_payment_terms::OutputFormat::Json,
                _ => commands::list_payment_terms::OutputFormat::Human,
            };
            commands::execute_list_payment_terms(tally_client, payee, &output_format).await
        }
    }
}

/// Execute agreement commands
async fn execute_agreement_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    command: &AgreementCommands,
) -> Result<String> {
    match command {
        AgreementCommands::List { payment_terms } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => commands::list_agreements::OutputFormat::Json,
                _ => commands::list_agreements::OutputFormat::Human,
            };
            commands::execute_list_agreements(tally_client, payment_terms, &output_format, config)
                .await
        }

        AgreementCommands::Show { agreement } => {
            let output_format = match cli.output {
                Some(OutputFormat::Json) => "json",
                _ => "human",
            };
            let request = commands::show_agreement::ShowAgreementRequest {
                agreement,
                output_format,
            };
            commands::execute_show_agreement(tally_client, &request, config).await
        }
    }
}

/// Execute dashboard commands
fn execute_dashboard_commands(
    cli: &Cli,
    tally_client: &SimpleTallyClient,
    config: &TallyCliConfig,
    config_file: &ConfigFile,
    command: &DashboardCommands,
) -> Result<String> {
    // Helper to resolve merchant from config when not provided
    let get_merchant = |merchant_opt: &Option<String>| -> Result<String> {
        merchant_opt.as_ref().map_or_else(
            || {
                config_file.active_profile().map_or_else(
                    || {
                        Err(anyhow::anyhow!(
                            "No active profile configured and merchant not provided.\n\
                             \n\
                             Initialize your configuration first:\n\
                                tally-merchant config init"
                        ))
                    },
                    |profile| {
                        profile.merchant.as_ref().map_or_else(
                            || {
                                Err(anyhow::anyhow!(
                                    "Merchant not provided and not configured in active profile.\n\
                                     \n\
                                     You can fix this by:\n\
                                     \n\
                                     1. Pass the merchant as an argument:\n\
                                        tally-merchant dashboard overview --merchant <MERCHANT_ADDRESS>\n\
                                     \n\
                                     2. Or configure it in your profile:\n\
                                        tally-merchant config set merchant <MERCHANT_ADDRESS>\n\
                                     \n\
                                     If you haven't created a merchant yet, run:\n\
                                        tally-merchant init"
                                ))
                            },
                            |merchant| Ok(merchant.clone()),
                        )
                    },
                )
            },
            |merchant| Ok(merchant.clone()),
        )
    };

    // Resolve merchant for commands that need it
    let command_with_merchant = match command {
        DashboardCommands::Overview { merchant } => {
            let merchant_addr = get_merchant(merchant)?;
            DashboardCommands::Overview {
                merchant: Some(merchant_addr),
            }
        }
        DashboardCommands::Events { merchant, since } => {
            let merchant_addr = get_merchant(merchant)?;
            DashboardCommands::Events {
                merchant: Some(merchant_addr),
                since: *since,
            }
        }
        DashboardCommands::Subscriptions {
            merchant,
            active_only,
        } => {
            let merchant_addr = get_merchant(merchant)?;
            DashboardCommands::Subscriptions {
                merchant: Some(merchant_addr),
                active_only: *active_only,
            }
        }
        DashboardCommands::Analytics { .. } => {
            // Analytics doesn't need merchant, use command as-is
            command.clone()
        }
    };

    let output_format = match cli.output {
        Some(OutputFormat::Json) => commands::dashboard::OutputFormat::Json,
        Some(OutputFormat::Csv) => commands::dashboard::OutputFormat::Csv,
        _ => commands::dashboard::OutputFormat::Human,
    };
    let rpc_url = cli.rpc_url.as_deref().unwrap_or(&config.default_rpc_url);
    commands::dashboard::execute(
        tally_client,
        &command_with_merchant,
        &output_format,
        rpc_url,
        config,
    )
}

/// Main command router
async fn execute_command(
    cli: &Cli,
    tally_client: Option<&SimpleTallyClient>,
    config: &TallyCliConfig,
    config_file: &ConfigFile,
) -> Result<String> {
    match &cli.command {
        Commands::Init { skip_plan } => {
            let client = require_client(tally_client)?;
            commands::execute_init_wizard(client, config, *skip_plan).await
        }
        Commands::Config { command } => {
            execute_config_commands(cli, tally_client, config, command).await
        }
        Commands::Payee { command } => {
            let client = require_client(tally_client)?;
            execute_payee_commands(cli, client, config, command).await
        }
        Commands::PaymentTerms { command } => {
            let client = require_client(tally_client)?;
            execute_payment_terms_commands(cli, client, config, command).await
        }
        Commands::Agreement { command } => {
            let client = require_client(tally_client)?;
            execute_agreement_commands(cli, client, config, command).await
        }
        Commands::Dashboard { command } => {
            let client = require_client(tally_client)?;
            execute_dashboard_commands(cli, client, config, config_file, command)
        }
        Commands::Completions {
            shell,
            install,
            yes,
            print,
            dry_run,
            uninstall,
        } => {
            use clap::CommandFactory;
            use commands::completions::CompletionAction;

            // Determine action based on flags (priority order)
            let action = if *print {
                CompletionAction::Print
            } else if *uninstall {
                CompletionAction::Uninstall
            } else if *dry_run {
                CompletionAction::DryRun
            } else if *install || *yes {
                CompletionAction::Install
            } else {
                CompletionAction::Auto
            };

            let args = commands::completions::CompletionsArgs {
                shell: *shell,
                action,
                skip_confirm: *yes,
            };
            let cmd = Cli::command();
            commands::completions::execute(&args, cmd)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usdc_to_micro_units_valid() {
        assert_eq!(usdc_to_micro_units(10.0).unwrap(), 10_000_000);
        assert_eq!(usdc_to_micro_units(0.5).unwrap(), 500_000);
        assert_eq!(usdc_to_micro_units(100.25).unwrap(), 100_250_000);
    }

    #[test]
    fn test_usdc_to_micro_units_zero() {
        assert_eq!(usdc_to_micro_units(0.0).unwrap(), 0);
    }

    #[test]
    fn test_usdc_to_micro_units_negative() {
        let result = usdc_to_micro_units(-1.0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("greater than or equal to 0"));
    }

    #[test]
    fn test_usdc_to_micro_units_too_large() {
        let result = usdc_to_micro_units(2_000_000.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("seems too high"));
    }

    #[test]
    fn test_usdc_to_micro_units_edge_case_max() {
        // Test maximum allowed value
        assert_eq!(usdc_to_micro_units(1_000_000.0).unwrap(), 1_000_000_000_000);
    }

    #[test]
    fn test_parse_output_format_human() {
        let result = parse_output_format("human").unwrap();
        assert!(matches!(result, OutputFormat::Human));
    }

    #[test]
    fn test_parse_output_format_json() {
        let result = parse_output_format("json").unwrap();
        assert!(matches!(result, OutputFormat::Json));
    }

    #[test]
    fn test_parse_output_format_csv() {
        let result = parse_output_format("csv").unwrap();
        assert!(matches!(result, OutputFormat::Csv));
    }

    #[test]
    fn test_parse_output_format_case_insensitive() {
        assert!(matches!(
            parse_output_format("Human").unwrap(),
            OutputFormat::Human
        ));
        assert!(matches!(
            parse_output_format("JSON").unwrap(),
            OutputFormat::Json
        ));
        assert!(matches!(
            parse_output_format("CSV").unwrap(),
            OutputFormat::Csv
        ));
        assert!(matches!(
            parse_output_format("HUMAN").unwrap(),
            OutputFormat::Human
        ));
    }

    #[test]
    fn test_parse_output_format_invalid() {
        let result = parse_output_format("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid output format"));
    }
}
