//! Progress indicators and spinners for long-running operations

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a spinner for indefinite progress (e.g., waiting for transaction confirmation)
///
/// # Panics
/// Panics if the progress bar template is invalid (should never happen with valid template)
#[must_use]
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(120));
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("Valid progress bar template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(message.to_string());
    spinner
}

/// Finish a progress indicator with a success message
pub fn finish_progress_success(progress: &ProgressBar, message: &str) {
    progress.finish_with_message(format!("✓ {message}"));
}

/// Finish a progress indicator with an error message
pub fn finish_progress_error(progress: &ProgressBar, message: &str) {
    progress.finish_with_message(format!("✗ {message}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_spinner() {
        let spinner = create_spinner("Test message");
        assert!(spinner.is_hidden() || !spinner.is_finished());
        spinner.finish();
    }

    #[test]
    fn test_finish_progress_success() {
        let progress = create_spinner("Test");
        finish_progress_success(&progress, "Success");
        assert!(progress.is_finished());
    }

    #[test]
    fn test_finish_progress_error() {
        let progress = create_spinner("Test");
        finish_progress_error(&progress, "Error");
        assert!(progress.is_finished());
    }
}
