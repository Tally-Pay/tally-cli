//! Shell completions installation and management
//!
//! Provides smart installation of shell completion scripts with support for
//! automatic installation, preview, and uninstallation.

use anyhow::{Context, Result};
use clap::Command;
use clap_complete::{generate, Shell};
use colored::Colorize;
use dialoguer::Confirm;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

/// Completions command arguments
///
/// Note: Multiple bools are allowed here as they represent independent CLI flags
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct CompletionsArgs {
    pub shell: Shell,
    pub install: bool,
    pub yes: bool,
    pub print: bool,
    pub dry_run: bool,
    pub uninstall: bool,
}

/// Execute the completions command with smart behavior
///
/// # Errors
///
/// Returns an error if:
/// - File I/O operations fail (creating directories, writing files)
/// - User's home directory cannot be determined
/// - Shell configuration files cannot be read or modified
pub fn execute(args: &CompletionsArgs, mut cmd: Command) -> Result<String> {
    // Handle explicit flags first
    if args.print {
        // Just print script to stdout (for piping/automation)
        print_completion_script(args.shell, &mut cmd);
        return Ok(String::new());
    }

    if args.uninstall {
        return uninstall_completions(args.shell, args.yes);
    }

    if args.dry_run {
        return show_installation_plan(args.shell);
    }

    if args.install || args.yes {
        return install_completions(args.shell, args.yes, &mut cmd);
    }

    // Default behavior: Smart based on TTY
    if io::stdout().is_terminal() {
        // Interactive mode: Show helpful installation guide
        show_installation_guide(args.shell, &mut cmd)
    } else {
        // Non-TTY (piped): Print script to stdout (preserves scriptability)
        print_completion_script(args.shell, &mut cmd);
        Ok(String::new())
    }
}

/// Print completion script to stdout (for piping)
fn print_completion_script(shell: Shell, cmd: &mut Command) {
    let bin_name = cmd.get_name().to_string();
    generate(shell, cmd, bin_name, &mut io::stdout());
}

/// Show interactive installation guide with option to install
fn show_installation_guide(shell: Shell, cmd: &mut Command) -> Result<String> {
    let config = get_shell_config(shell)?;
    let bin_name = "tally-merchant";

    let mut output = String::new();
    writeln!(output, "\n{}\n", "Shell Completion Setup".bright_cyan().bold())?;

    writeln!(output, "Installing completions for {}:\n", shell.to_string().bright_green())?;

    output.push_str("This will:\n");
    writeln!(output, "  {} Create completion directory: {}", "•".bright_blue(), config.completion_dir.display())?;
    writeln!(output, "  {} Write completion script: {}", "•".bright_blue(), config.completion_file.display())?;
    if let Some(rc_file) = &config.rc_file {
        writeln!(output, "  {} Update shell config: {}", "•".bright_blue(), rc_file.display())?;
    }
    output.push('\n');

    println!("{output}");

    // Prompt for installation
    let should_install = Confirm::new()
        .with_prompt("Install completions now?")
        .default(true)
        .interact()?;

    if should_install {
        install_completions(shell, false, cmd)
    } else {
        Ok(format!(
            "\n{}\n\nTo install later, run:\n  {} completions {} --install\n\nTo see the completion script:\n  {} completions {} --print\n",
            "Installation skipped".yellow(),
            bin_name,
            shell,
            bin_name,
            shell
        ))
    }
}

/// Install completions automatically
fn install_completions(shell: Shell, skip_confirm: bool, cmd: &mut Command) -> Result<String> {
    let config = get_shell_config(shell)?;
    let bin_name = "tally-merchant";

    // Check if already installed
    if config.completion_file.exists() && !skip_confirm {
        let should_overwrite = Confirm::new()
            .with_prompt(format!(
                "Completions already installed at {}. Overwrite?",
                config.completion_file.display()
            ))
            .default(false)
            .interact()?;

        if !should_overwrite {
            return Ok(format!("{} Installation canceled", "✗".red()));
        }
    }

    // Create completion directory if it doesn't exist
    if !config.completion_dir.exists() {
        fs::create_dir_all(&config.completion_dir)
            .with_context(|| format!("Failed to create directory: {}", config.completion_dir.display()))?;
    }

    // Generate completion script
    let mut completion_script = Vec::new();
    {
        let bin_name_str = cmd.get_name().to_string();
        generate(shell, cmd, bin_name_str, &mut completion_script);
    }

    // Write completion file
    fs::write(&config.completion_file, &completion_script)
        .with_context(|| format!("Failed to write completion file: {}", config.completion_file.display()))?;

    // Update shell RC file if needed
    let mut rc_modified = false;
    if let Some(rc_file) = &config.rc_file {
        if let Some(rc_line) = &config.rc_line_to_add {
            rc_modified = ensure_rc_line(rc_file, rc_line)?;
        }
    }

    // Build success message
    let mut output = String::new();
    writeln!(output, "\n{}\n", "✓ Completions installed successfully".bright_green().bold())?;

    writeln!(output, "Location: {}", config.completion_file.display().to_string().bright_white())?;

    if rc_modified {
        if let Some(rc_file) = &config.rc_file {
            writeln!(output, "Modified: {}", rc_file.display().to_string().bright_white())?;
        }
    }

    writeln!(output, "\n{}", "Next steps:".bright_cyan())?;
    writeln!(output, "  1. Restart your shell, or run: {}", config.reload_command.bright_yellow())?;
    writeln!(output, "  2. Test with: {} {}", bin_name.bright_yellow(), "<TAB>".bright_yellow())?;

    writeln!(output, "\n{}", "Troubleshooting:".bright_cyan())?;
    writeln!(output, "  • Completions not working? Check: {}", config.troubleshooting_command.bright_white())?;
    writeln!(output, "  • Need to reinstall? Run: {bin_name} completions {shell} --install --yes")?;
    writeln!(output, "  • Uninstall: {bin_name} completions {shell} --uninstall")?;

    Ok(output)
}

/// Show what would be installed without making changes
fn show_installation_plan(shell: Shell) -> Result<String> {
    let config = get_shell_config(shell)?;

    let mut output = String::new();
    writeln!(output, "\n{}\n", "Installation Plan (dry run)".bright_cyan().bold())?;

    writeln!(output, "Shell: {}\n", shell.to_string().bright_green())?;

    output.push_str("Will install to:\n");
    writeln!(output, "  {}\n", config.completion_file.display())?;

    if let Some(rc_file) = &config.rc_file {
        writeln!(output, "Will modify: {}", rc_file.display())?;
        if let Some(rc_line) = &config.rc_line_to_add {
            output.push_str("Changes:\n");
            writeln!(output, "  {} {}", "+".bright_green(), rc_line)?;
        }
        output.push('\n');
    }

    if config.completion_dir.exists() {
        writeln!(output, "{} Directory exists: {}", "✓".bright_green(), config.completion_dir.display())?;
    } else {
        writeln!(output, "{} Will create directory: {}", "•".bright_blue(), config.completion_dir.display())?;
    }

    if config.completion_file.exists() {
        writeln!(output, "{} Will overwrite existing file", "⚠".bright_yellow())?;
    }

    writeln!(output, "\nTo install, run:\n  tally-merchant completions {shell} --install")?;

    Ok(output)
}

/// Uninstall completions
fn uninstall_completions(shell: Shell, skip_confirm: bool) -> Result<String> {
    let config = get_shell_config(shell)?;

    if !config.completion_file.exists() {
        return Ok(format!("{} Completions are not installed", "ℹ".bright_blue()));
    }

    if !skip_confirm {
        let should_uninstall = Confirm::new()
            .with_prompt(format!(
                "Remove completion file at {}?",
                config.completion_file.display()
            ))
            .default(true)
            .interact()?;

        if !should_uninstall {
            return Ok(format!("{} Uninstallation canceled", "✗".red()));
        }
    }

    // Remove completion file
    fs::remove_file(&config.completion_file)
        .with_context(|| format!("Failed to remove completion file: {}", config.completion_file.display()))?;

    let mut output = String::new();
    writeln!(output, "\n{}\n", "✓ Completions uninstalled successfully".bright_green().bold())?;
    writeln!(output, "Removed: {}", config.completion_file.display().to_string().bright_white())?;

    if let Some(rc_file) = &config.rc_file {
        writeln!(output, "\n{} The following line in {} was not removed:", "ℹ".bright_blue(), rc_file.display())?;
        if let Some(rc_line) = &config.rc_line_to_add {
            writeln!(output, "  {}", rc_line.bright_white())?;
        }
        output.push_str("\nYou may want to remove it manually if no longer needed.\n");
    }

    writeln!(output, "\nRestart your shell or run: {}", config.reload_command.bright_yellow())?;

    Ok(output)
}

/// Shell-specific configuration
struct ShellConfig {
    completion_dir: PathBuf,
    completion_file: PathBuf,
    rc_file: Option<PathBuf>,
    rc_line_to_add: Option<String>,
    reload_command: String,
    troubleshooting_command: String,
}

/// Get shell-specific paths and configuration
fn get_shell_config(shell: Shell) -> Result<ShellConfig> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let bin_name = "tally-merchant";

    match shell {
        Shell::Zsh => {
            let completion_dir = home.join(".zsh").join("completions");
            let completion_file = completion_dir.join(format!("_{bin_name}"));
            let rc_file = Some(home.join(".zshrc"));
            let rc_line_to_add = Some(format!("fpath=({}/.zsh/completions $fpath)", home.display()));

            Ok(ShellConfig {
                completion_dir,
                completion_file,
                rc_file,
                rc_line_to_add,
                reload_command: "source ~/.zshrc".to_string(),
                troubleshooting_command: "echo $fpath".to_string(),
            })
        }
        Shell::Bash => {
            let completion_dir = home.join(".bash_completion.d");
            let completion_file = completion_dir.join(bin_name);
            let rc_file = Some(home.join(".bashrc"));
            let rc_line_to_add = Some(format!(
                "[ -f ~/.bash_completion.d/{bin_name} ] && source ~/.bash_completion.d/{bin_name}"
            ));

            Ok(ShellConfig {
                completion_dir,
                completion_file,
                rc_file,
                rc_line_to_add,
                reload_command: "source ~/.bashrc".to_string(),
                troubleshooting_command: "echo $BASH_COMPLETION_COMPAT_DIR".to_string(),
            })
        }
        Shell::Fish => {
            let completion_dir = home.join(".config").join("fish").join("completions");
            let completion_file = completion_dir.join(format!("{bin_name}.fish"));

            Ok(ShellConfig {
                completion_dir,
                completion_file,
                rc_file: None, // Fish auto-loads from completions directory
                rc_line_to_add: None,
                reload_command: "exec fish".to_string(),
                troubleshooting_command: "echo $fish_complete_path".to_string(),
            })
        }
        Shell::PowerShell => {
            let completion_dir = home.join(".config").join("powershell").join("completions");
            let completion_file = completion_dir.join(format!("{bin_name}.ps1"));

            Ok(ShellConfig {
                completion_dir,
                completion_file,
                rc_file: None, // PowerShell profile location is complex
                rc_line_to_add: None,
                reload_command: ". $PROFILE".to_string(),
                troubleshooting_command: "echo $PROFILE".to_string(),
            })
        }
        Shell::Elvish => {
            let completion_dir = home.join(".config").join("elvish").join("completions");
            let completion_file = completion_dir.join(format!("{bin_name}.elv"));

            Ok(ShellConfig {
                completion_dir,
                completion_file,
                rc_file: None,
                rc_line_to_add: None,
                reload_command: "exec elvish".to_string(),
                troubleshooting_command: "echo $paths".to_string(),
            })
        }
        _ => Err(anyhow::anyhow!(
            "Shell {shell} is not supported for automatic installation"
        )),
    }
}

/// Ensure a line exists in an RC file (idempotent)
fn ensure_rc_line(rc_file: &PathBuf, line_to_add: &str) -> Result<bool> {
    // Read existing content
    let content = if rc_file.exists() {
        fs::read_to_string(rc_file)
            .with_context(|| format!("Failed to read {}", rc_file.display()))?
    } else {
        String::new()
    };

    // Check if line already exists
    if content.lines().any(|line| line.trim() == line_to_add.trim()) {
        return Ok(false); // Already exists, no modification needed
    }

    // For zsh, detect framework initialization and insert before it
    let new_content = if rc_file.ends_with(".zshrc") {
        insert_before_zsh_framework(&content, line_to_add)
    } else {
        // For other shells, append at the end
        append_line(&content, line_to_add)
    };

    fs::write(rc_file, new_content)
        .with_context(|| format!("Failed to write {}", rc_file.display()))?;

    Ok(true) // Modified
}

/// Insert line before zsh framework initialization, or append if no framework found
fn insert_before_zsh_framework(content: &str, line_to_add: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Patterns that indicate framework source/init (must come before fpath)
    let framework_patterns = [
        "source $ZSH/oh-my-zsh.sh",
        "source ${ZSH}/oh-my-zsh.sh",
        "source ~/.oh-my-zsh/oh-my-zsh.sh",
        "source $HOME/.oh-my-zsh/oh-my-zsh.sh",
        "source \"${ZDOTDIR:-$HOME}/.zprezto/init.zsh\"",
        "zinit light",
        "antigen apply",
    ];

    // Find the first framework initialization line
    lines.iter().position(|line| {
        let trimmed = line.trim();
        framework_patterns.iter().any(|pattern| trimmed.contains(pattern))
    }).map_or_else(
        || {
            // No framework found, append at the end
            append_line(content, line_to_add)
        },
        |pos| {
            // Insert before the framework line with a comment
            let mut result = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i == pos {
                    result.push_str("# Custom completion directories (must be before framework init)\n");
                    result.push_str(line_to_add);
                    result.push('\n');
                }
                result.push_str(line);
                result.push('\n');
            }
            result
        }
    )
}

/// Append line to content
fn append_line(content: &str, line_to_add: &str) -> String {
    if content.is_empty() || content.ends_with('\n') {
        format!("{content}{line_to_add}\n")
    } else {
        format!("{content}\n{line_to_add}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shell_config_zsh() {
        let config = get_shell_config(Shell::Zsh).unwrap();
        assert!(config.completion_file.ends_with("_tally-merchant"));
        assert!(config.rc_file.is_some());
        assert!(config.rc_line_to_add.is_some());
    }

    #[test]
    fn test_get_shell_config_bash() {
        let config = get_shell_config(Shell::Bash).unwrap();
        assert!(config.completion_file.ends_with("tally-merchant"));
        assert!(config.rc_file.is_some());
        assert!(config.rc_line_to_add.is_some());
    }

    #[test]
    fn test_get_shell_config_fish() {
        let config = get_shell_config(Shell::Fish).unwrap();
        assert!(config.completion_file.ends_with("tally-merchant.fish"));
        assert!(config.rc_file.is_none()); // Fish auto-loads
        assert!(config.rc_line_to_add.is_none());
    }

    #[test]
    fn test_insert_before_oh_my_zsh() {
        let content = "export ZSH=\"$HOME/.oh-my-zsh\"\nplugins=(git)\n\nsource $ZSH/oh-my-zsh.sh\n\necho 'done'\n";
        let line_to_add = "fpath=(~/.zsh/completions $fpath)";

        let result = insert_before_zsh_framework(content, line_to_add);

        // Should insert before the source line
        assert!(result.contains("# Custom completion directories"));
        assert!(result.contains("fpath=(~/.zsh/completions $fpath)"));

        // Verify it's before the source line
        let source_pos = result.find("source $ZSH/oh-my-zsh.sh").unwrap();
        let fpath_pos = result.find("fpath=(~/.zsh/completions $fpath)").unwrap();
        assert!(fpath_pos < source_pos, "fpath should be before source");
    }

    #[test]
    fn test_insert_no_framework() {
        let content = "export PATH=$HOME/bin:$PATH\nalias ll='ls -la'\n";
        let line_to_add = "fpath=(~/.zsh/completions $fpath)";

        let result = insert_before_zsh_framework(content, line_to_add);

        // Should append at the end since no framework found
        assert!(result.ends_with("fpath=(~/.zsh/completions $fpath)\n"));
    }

    #[test]
    fn test_append_line_empty_content() {
        let result = append_line("", "test line");
        assert_eq!(result, "test line\n");
    }

    #[test]
    fn test_append_line_with_newline() {
        let result = append_line("existing content\n", "test line");
        assert_eq!(result, "existing content\ntest line\n");
    }

    #[test]
    fn test_append_line_without_newline() {
        let result = append_line("existing content", "test line");
        assert_eq!(result, "existing content\ntest line\n");
    }
}
