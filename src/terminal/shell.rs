//! Shell execution utilities

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use anyhow::Result;

/// Shell wrapper for command execution
pub struct Shell {
    /// Current working directory
    cwd: PathBuf,
    /// Running process
    process: Option<Child>,
    /// Output buffer
    output_buffer: Vec<String>,
    /// Input history
    input_history: Vec<String>,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    /// Create a new shell instance
    pub fn new() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_default(),
            process: None,
            output_buffer: Vec::new(),
            input_history: Vec::new(),
        }
    }

    /// Create a shell with a specific working directory
    #[allow(dead_code)]
    pub fn with_cwd(cwd: PathBuf) -> Self {
        Self {
            cwd,
            process: None,
            output_buffer: Vec::new(),
            input_history: Vec::new(),
        }
    }

    /// Get the current working directory
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    /// Set the current working directory
    pub fn set_cwd(&mut self, cwd: PathBuf) -> Result<()> {
        if cwd.is_dir() {
            self.cwd = cwd.canonicalize()?;
            Ok(())
        } else {
            anyhow::bail!("Not a directory: {}", cwd.display())
        }
    }

    /// Execute a command and capture output
    pub fn execute(&mut self, command: &str) -> Result<Vec<String>> {
        self.input_history.push(command.to_string());

        #[cfg(windows)]
        let mut child = Command::new("cmd")
            .args(["/C", command])
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        #[cfg(not(windows))]
        let mut child = Command::new("sh")
            .args(["-c", command])
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut output = Vec::new();

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                output.push(line);
            }
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                output.push(format!("[stderr] {}", line));
            }
        }

        // Wait for completion
        let status = child.wait()?;
        if !status.success() {
            output.push(format!("[exit code: {:?}]", status.code()));
        }

        self.output_buffer.extend(output.clone());

        Ok(output)
    }

    /// Execute a command asynchronously (non-blocking)
    #[allow(dead_code)]
    pub fn execute_async(&mut self, command: &str) -> Result<()> {
        #[cfg(windows)]
        let child = Command::new("cmd")
            .args(["/C", command])
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        #[cfg(not(windows))]
        let child = Command::new("sh")
            .args(["-c", command])
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.process = Some(child);
        Ok(())
    }

    /// Read output from async process
    #[allow(dead_code)]
    pub fn read_output(&mut self) -> Option<String> {
        if let Some(ref mut child) = self.process {
            if let Some(ref mut stdout) = child.stdout {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => None,
                    Ok(_) => Some(line),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if async process is still running
    #[allow(dead_code)]
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.process = None;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Kill the running process
    #[allow(dead_code)]
    pub fn kill(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.process {
            child.kill()?;
            self.process = None;
        }
        Ok(())
    }

    /// Get the output buffer
    #[allow(dead_code)]
    pub fn output(&self) -> &[String] {
        &self.output_buffer
    }

    /// Clear the output buffer
    #[allow(dead_code)]
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }

    /// Get command history
    #[allow(dead_code)]
    pub fn history(&self) -> &[String] {
        &self.input_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_execute() {
        let mut shell = Shell::new();

        #[cfg(windows)]
        let output = shell.execute("echo hello").unwrap();

        #[cfg(not(windows))]
        let output = shell.execute("echo hello").unwrap();

        assert!(!output.is_empty());
    }
}
