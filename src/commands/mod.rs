//! Command implementations for the Tally CLI
//!
//! This module contains the individual command implementations, each in their own file
//! for better organization and maintainability.

pub mod config_file_ops;
pub mod create_plan;
pub mod dashboard;
pub mod deactivate_plan;
pub mod init_merchant;
pub mod init_wizard;
pub mod list_plans;
pub mod list_subs;
pub mod show_config;
pub mod show_merchant;
pub mod show_subscription;
pub mod update_plan_terms;

// Re-export command execution functions for easy access
pub use create_plan::execute as execute_create_plan;
pub use deactivate_plan::execute as execute_deactivate_plan;
pub use init_merchant::execute as execute_init_merchant;
pub use init_wizard::execute as execute_init_wizard;
pub use list_plans::execute as execute_list_plans;
pub use list_subs::execute as execute_list_subs;
pub use show_config::execute as execute_show_config;
pub use show_merchant::execute as execute_show_merchant;
pub use show_subscription::execute as execute_show_subscription;
pub use update_plan_terms::execute as execute_update_plan_terms;
