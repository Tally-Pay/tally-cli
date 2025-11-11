//! Color theme and terminal output utilities for the Tally CLI
//!
//! Respects `NO_COLOR` environment variable and --no-color CLI flag.

use colored::{ColoredString, Colorize};
use std::sync::OnceLock;

/// Global color state - initialized once based on environment and CLI flags
static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();

/// Initialize color support based on environment and CLI flags
pub fn init_colors(no_color_flag: bool) {
    let should_enable =
        !no_color_flag && std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout);

    COLOR_ENABLED.get_or_init(|| should_enable);

    // Disable colored crate if colors are disabled
    if !should_enable {
        colored::control::set_override(false);
    }
}

/// Check if colors are enabled
#[must_use]
pub fn colors_enabled() -> bool {
    *COLOR_ENABLED
        .get_or_init(|| std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout))
}

/// Color theme for CLI output
pub struct Theme;

impl Theme {
    /// Success message color (green)
    #[must_use]
    pub fn success(text: &str) -> ColoredString {
        if colors_enabled() {
            text.green().bold()
        } else {
            text.normal()
        }
    }

    /// Error message color (red)
    #[must_use]
    pub fn error(text: &str) -> ColoredString {
        if colors_enabled() {
            text.red().bold()
        } else {
            text.normal()
        }
    }

    /// Warning message color (yellow)
    #[must_use]
    pub fn warning(text: &str) -> ColoredString {
        if colors_enabled() {
            text.yellow().bold()
        } else {
            text.normal()
        }
    }

    /// Info message color (blue)
    #[must_use]
    pub fn info(text: &str) -> ColoredString {
        if colors_enabled() {
            text.blue()
        } else {
            text.normal()
        }
    }

    /// Highlight important text (cyan)
    #[must_use]
    pub fn highlight(text: &str) -> ColoredString {
        if colors_enabled() {
            text.cyan().bold()
        } else {
            text.normal()
        }
    }

    /// Table header color (bold white)
    #[must_use]
    pub fn header(text: &str) -> ColoredString {
        if colors_enabled() {
            text.white().bold()
        } else {
            text.normal()
        }
    }

    /// Dim text for secondary information (dark gray)
    #[must_use]
    pub fn dim(text: &str) -> ColoredString {
        if colors_enabled() {
            text.bright_black()
        } else {
            text.normal()
        }
    }

    /// Active/enabled status (green)
    #[must_use]
    pub fn active(text: &str) -> ColoredString {
        if colors_enabled() {
            text.green()
        } else {
            text.normal()
        }
    }

    /// Inactive/disabled status (red)
    #[must_use]
    pub fn inactive(text: &str) -> ColoredString {
        if colors_enabled() {
            text.red()
        } else {
            text.normal()
        }
    }

    /// Format a value with color (magenta for values)
    #[must_use]
    pub fn value(text: &str) -> ColoredString {
        if colors_enabled() {
            text.magenta()
        } else {
            text.normal()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_methods_exist() {
        // Basic smoke test to ensure Theme methods compile
        let _ = Theme::header("test");
        let _ = Theme::info("test");
        let _ = Theme::value("test");
        let _ = Theme::warning("test");
        let _ = Theme::error("test");
    }
}
