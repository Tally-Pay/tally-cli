//! Command implementations for the Tally CLI
//!
//! This module contains the individual command implementations, each in their own file
//! for better organization and maintainability.

pub mod cancel_subscription;
pub mod close_subscription;
pub mod create_plan;
pub mod dashboard;
pub mod deactivate_plan;
pub mod init_config;
pub mod init_merchant;
pub mod list_plans;
pub mod list_subs;
pub mod renew_subscription;
pub mod show_config;
pub mod show_merchant;
pub mod show_subscription;
pub mod simulate_events;
pub mod start_subscription;
pub mod update_plan_terms;
pub mod withdraw_fees;

// Re-export command execution functions for easy access
pub use cancel_subscription::execute as execute_cancel_subscription;
pub use close_subscription::execute as execute_close_subscription;
pub use create_plan::execute as execute_create_plan;
pub use dashboard::execute as execute_dashboard_command;
pub use deactivate_plan::execute as execute_deactivate_plan;
pub use init_config::execute as execute_init_config;
pub use init_merchant::execute as execute_init_merchant;
pub use list_plans::execute as execute_list_plans;
pub use list_subs::execute as execute_list_subs;
pub use renew_subscription::execute as execute_renew_subscription;
pub use show_config::execute as execute_show_config;
pub use show_merchant::execute as execute_show_merchant;
pub use show_subscription::execute as execute_show_subscription;
pub use simulate_events::execute as execute_simulate_events;
pub use start_subscription::execute as execute_start_subscription;
pub use update_plan_terms::execute as execute_update_plan_terms;
pub use withdraw_fees::execute as execute_withdraw_fees;
