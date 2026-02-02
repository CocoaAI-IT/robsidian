//! Terminal functionality for command execution

pub mod ansi;
pub mod buffer;
pub mod pty;
pub mod shell;

pub use ansi::AnsiParser;
pub use buffer::{CursorPos, StyledChar, TerminalBuffer, TerminalLine};
pub use pty::{PtyTerminal, TerminalKey};

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

/// Terminal tab state
#[derive(Debug, Clone)]
pub struct TerminalTab {
    /// Output lines
    pub output: Vec<String>,
    /// Command history
    pub history: Vec<String>,
    /// Current working directory
    pub cwd: std::path::PathBuf,
}

impl Default for TerminalTab {
    fn default() -> Self {
        Self {
            output: Vec::new(),
            history: Vec::new(),
            cwd: std::env::current_dir().unwrap_or_default(),
        }
    }
}

/// Terminal state managing multiple tabs
#[derive(Debug)]
pub struct TerminalState {
    /// Terminal tabs
    pub tabs: Vec<TerminalTab>,
    /// Active tab index
    pub active_tab: usize,
    /// Current input
    pub input: String,
    /// History index for navigation
    history_index: Option<usize>,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalState {
    /// Create a new terminal state
    pub fn new() -> Self {
        Self {
            tabs: vec![TerminalTab::default()],
            active_tab: 0,
            input: String::new(),
            history_index: None,
        }
    }

    /// Create a new terminal tab
    pub fn new_tab(&mut self) {
        self.tabs.push(TerminalTab::default());
        self.active_tab = self.tabs.len() - 1;
    }

    /// Close the current tab
    #[allow(dead_code)]
    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    /// Get the current tab
    pub fn current_tab(&self) -> Option<&TerminalTab> {
        self.tabs.get(self.active_tab)
    }

    /// Get the current tab mutably
    pub fn current_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        self.tabs.get_mut(self.active_tab)
    }

    /// Execute the current input command
    pub fn execute_command(&mut self) {
        let command = self.input.trim().to_string();
        if command.is_empty() {
            return;
        }

        // Add to history
        if let Some(tab) = self.current_tab_mut() {
            tab.history.push(command.clone());
            tab.output.push(format!("$ {}", command));
        }

        // Clear input
        self.input.clear();
        self.history_index = None;

        // Handle built-in commands
        if command.starts_with("cd ") {
            self.handle_cd(&command[3..]);
            return;
        }

        if command == "clear" || command == "cls" {
            self.clear_output();
            return;
        }

        // Execute external command
        self.run_command(&command);
    }

    /// Handle cd command
    fn handle_cd(&mut self, path: &str) {
        let path = path.trim();
        let new_path = if path == "~" {
            dirs::home_dir().unwrap_or_default()
        } else if let Some(tab) = self.current_tab() {
            if std::path::Path::new(path).is_absolute() {
                std::path::PathBuf::from(path)
            } else {
                tab.cwd.join(path)
            }
        } else {
            return;
        };

        if new_path.is_dir() {
            if let Some(tab) = self.current_tab_mut() {
                tab.cwd = new_path.canonicalize().unwrap_or(new_path);
                tab.output.push(format!("Changed to: {}", tab.cwd.display()));
            }
        } else {
            if let Some(tab) = self.current_tab_mut() {
                tab.output.push(format!("Directory not found: {}", path));
            }
        }
    }

    /// Run an external command
    fn run_command(&mut self, command: &str) {
        let cwd = self.current_tab().map(|t| t.cwd.clone()).unwrap_or_default();

        // Use cmd on Windows, sh on Unix
        #[cfg(windows)]
        let result = Command::new("cmd")
            .args(["/C", command])
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        #[cfg(not(windows))]
        let result = Command::new("sh")
            .args(["-c", command])
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                // Read stdout
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        if let Some(tab) = self.current_tab_mut() {
                            tab.output.push(line);
                        }
                    }
                }

                // Read stderr
                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().map_while(Result::ok) {
                        if let Some(tab) = self.current_tab_mut() {
                            tab.output.push(format!("[stderr] {}", line));
                        }
                    }
                }

                // Wait for completion
                match child.wait() {
                    Ok(status) => {
                        if !status.success() {
                            if let Some(tab) = self.current_tab_mut() {
                                tab.output.push(format!("Exit code: {:?}", status.code()));
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(tab) = self.current_tab_mut() {
                            tab.output.push(format!("Process error: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(tab) = self.current_tab_mut() {
                    tab.output.push(format!("Failed to execute: {}", e));
                }
            }
        }
    }

    /// Clear output
    pub fn clear_output(&mut self) {
        if let Some(tab) = self.current_tab_mut() {
            tab.output.clear();
        }
    }

    /// Navigate history up
    #[allow(dead_code)]
    pub fn history_up(&mut self) {
        let Some(tab) = self.tabs.get(self.active_tab) else {
            return;
        };

        if tab.history.is_empty() {
            return;
        }

        let history_len = tab.history.len();
        let new_index = match self.history_index {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => history_len - 1,
        };

        let new_input = tab.history[new_index].clone();
        self.history_index = Some(new_index);
        self.input = new_input;
    }

    /// Navigate history down
    #[allow(dead_code)]
    pub fn history_down(&mut self) {
        let Some(tab) = self.tabs.get(self.active_tab) else {
            return;
        };

        let Some(i) = self.history_index else {
            return;
        };

        let history_len = tab.history.len();
        if i < history_len - 1 {
            let new_input = tab.history[i + 1].clone();
            self.history_index = Some(i + 1);
            self.input = new_input;
        } else {
            self.history_index = None;
            self.input.clear();
        }
    }
}

/// Helper module for home directory
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var("USERPROFILE").ok().map(PathBuf::from)
        }
        #[cfg(not(windows))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
    }
}

/// PTY-based terminal tab for interactive shell sessions
pub struct PtyTerminalTab {
    /// The PTY terminal instance
    pub pty: PtyTerminal,
    /// Terminal buffer for rendering
    pub buffer: TerminalBuffer,
    /// ANSI parser for processing output
    pub parser: AnsiParser,
    /// Error message if shell failed to start
    pub error: Option<String>,
}

impl PtyTerminalTab {
    /// Create a new PTY terminal tab with Nushell
    pub fn new_nushell() -> Self {
        Self::new_shell("nu")
    }

    /// Create a new PTY terminal tab with PowerShell
    pub fn new_powershell() -> Self {
        #[cfg(windows)]
        {
            Self::new_shell("pwsh")
        }
        #[cfg(not(windows))]
        {
            Self::new_shell("pwsh")
        }
    }

    /// Create a new PTY terminal tab with the default shell
    pub fn new_default() -> Self {
        #[cfg(windows)]
        {
            Self::new_shell("cmd")
        }
        #[cfg(not(windows))]
        {
            Self::new_shell("bash")
        }
    }

    /// Create a new PTY terminal tab with specified shell
    pub fn new_shell(shell: &str) -> Self {
        match PtyTerminal::new_shell(shell) {
            Ok(pty) => {
                let (cols, rows) = pty.size();
                Self {
                    pty,
                    buffer: TerminalBuffer::new(cols, rows),
                    parser: AnsiParser::new(),
                    error: None,
                }
            }
            Err(e) => {
                // Return a tab with error state
                Self {
                    pty: PtyTerminal::new_shell("cmd").unwrap_or_else(|_| {
                        panic!("Failed to create fallback shell")
                    }),
                    buffer: TerminalBuffer::new(80, 24),
                    parser: AnsiParser::new(),
                    error: Some(format!("Failed to start {}: {}", shell, e)),
                }
            }
        }
    }

    /// Process pending output from the PTY
    pub fn process_output(&mut self) {
        let output = self.pty.read_output();
        if !output.is_empty() {
            self.parser.process(&output, &mut self.buffer);
        }
    }

    /// Write input to the PTY
    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.pty.write(data)
    }

    /// Send a special key
    pub fn send_key(&mut self, key: TerminalKey) -> anyhow::Result<()> {
        self.pty.send_key(key)
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.pty.resize(cols, rows)?;
        self.buffer.resize(cols, rows);
        Ok(())
    }

    /// Check if the shell is still running
    pub fn is_alive(&mut self) -> bool {
        self.pty.is_alive()
    }
}

/// PTY-based terminal state with multiple tabs
pub struct PtyTerminalState {
    /// Terminal tabs
    pub tabs: Vec<PtyTerminalTab>,
    /// Active tab index
    pub active_tab: usize,
    /// Default shell to use for new tabs
    pub default_shell: String,
}

impl Default for PtyTerminalState {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyTerminalState {
    /// Create a new PTY terminal state with one Nushell tab
    pub fn new() -> Self {
        Self {
            tabs: vec![PtyTerminalTab::new_nushell()],
            active_tab: 0,
            default_shell: "nu".to_string(),
        }
    }

    /// Create a new PTY terminal state with specified shell
    pub fn with_shell(shell: &str) -> Self {
        Self {
            tabs: vec![PtyTerminalTab::new_shell(shell)],
            active_tab: 0,
            default_shell: shell.to_string(),
        }
    }

    /// Create a new tab with the default shell
    pub fn new_tab(&mut self) {
        self.tabs.push(PtyTerminalTab::new_shell(&self.default_shell));
        self.active_tab = self.tabs.len() - 1;
    }

    /// Create a new tab with specific shell
    pub fn new_tab_with_shell(&mut self, shell: &str) {
        self.tabs.push(PtyTerminalTab::new_shell(shell));
        self.active_tab = self.tabs.len() - 1;
    }

    /// Close the current tab
    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    /// Get the current tab
    pub fn current_tab(&self) -> Option<&PtyTerminalTab> {
        self.tabs.get(self.active_tab)
    }

    /// Get the current tab mutably
    pub fn current_tab_mut(&mut self) -> Option<&mut PtyTerminalTab> {
        self.tabs.get_mut(self.active_tab)
    }

    /// Switch to a specific tab
    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    /// Process output for all tabs
    pub fn process_all_output(&mut self) {
        for tab in &mut self.tabs {
            tab.process_output();
        }
    }

    /// Process output for current tab only
    pub fn process_current_output(&mut self) {
        if let Some(tab) = self.current_tab_mut() {
            tab.process_output();
        }
    }
}
