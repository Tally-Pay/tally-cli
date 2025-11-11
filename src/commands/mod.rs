//! Command implementations for the Tally CLI
//!
//! This module contains the individual command implementations, each in their own file
//! for better organization and maintainability.

pub mod completions;
pub mod config_file_ops;
pub mod create_payment_terms;
pub mod dashboard;
pub mod init_payee;
pub mod init_wizard;
pub mod list_agreements;
pub mod list_payment_terms;
pub mod show_agreement;
pub mod show_config;
pub mod show_payee;

// Re-export command execution functions for easy access
pub use create_payment_terms::execute as execute_create_payment_terms;
pub use init_payee::execute as execute_init_payee;
pub use init_wizard::execute as execute_init_wizard;
pub use list_agreements::execute as execute_list_agreements;
pub use list_payment_terms::execute as execute_list_payment_terms;
pub use show_agreement::execute as execute_show_agreement;
pub use show_config::execute as execute_show_config;
pub use show_payee::execute as execute_show_payee;
