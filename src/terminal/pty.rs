//! PTY management for embedded terminal
//!
//! This module provides PTY (pseudo-terminal) management using portable-pty,
//! allowing Nushell or other shells to be embedded within the application.

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

/// A PTY-based terminal that manages a shell subprocess
pub struct PtyTerminal {
    child: Box<dyn Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    output_rx: Receiver<Vec<u8>>,
    size: PtySize,
    shell_name: String,
}

impl PtyTerminal {
    /// Create a new PTY terminal with Nushell
    pub fn new_nushell() -> Result<Self> {
        Self::new_shell("nu")
    }

    /// Create a new PTY terminal with the specified shell
    pub fn new_shell(shell: &str) -> Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair: PtyPair = pty_system
            .openpty(size)
            .context("Failed to open PTY pair")?;

        let mut cmd = CommandBuilder::new(shell);

        // Set environment variables for better terminal experience
        cmd.env("TERM", "xterm-256color");

        // For Nushell, disable some features that don't work well in embedded terminals
        if shell == "nu" {
            cmd.env("NO_COLOR", "0"); // Allow colors
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .context(format!("Failed to spawn {}. Is it installed?", shell))?;

        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;

        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY writer")?;

        // Set up non-blocking output reading via channel
        let (output_tx, output_rx) = mpsc::channel();
        Self::spawn_reader_thread(reader, output_tx);

        Ok(Self {
            child,
            writer,
            output_rx,
            size,
            shell_name: shell.to_string(),
        })
    }

    /// Spawn a background thread to read PTY output
    fn spawn_reader_thread(mut reader: Box<dyn Read + Send>, tx: Sender<Vec<u8>>) {
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(e) => {
                        tracing::error!("PTY read error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    /// Write data to the PTY (send to shell)
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer
            .write_all(data)
            .context("Failed to write to PTY")?;
        self.writer.flush().context("Failed to flush PTY writer")?;
        Ok(())
    }

    /// Write a string to the PTY
    pub fn write_str(&mut self, s: &str) -> Result<()> {
        self.write(s.as_bytes())
    }

    /// Read available output from the PTY (non-blocking)
    pub fn read_output(&self) -> Vec<u8> {
        let mut output = Vec::new();
        loop {
            match self.output_rx.try_recv() {
                Ok(data) => output.extend(data),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        output
    }

    /// Resize the PTY
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        // Note: portable-pty's resize is on the pair.master, which we don't store
        // For now, we track the size but can't actually resize
        // TODO: Store master reference for resize support
        Ok(())
    }

    /// Get the current size
    pub fn size(&self) -> (u16, u16) {
        (self.size.cols, self.size.rows)
    }

    /// Check if the child process is still running
    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false, // Process exited
            Ok(None) => true,      // Still running
            Err(_) => false,       // Error checking, assume dead
        }
    }

    /// Kill the child process
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().context("Failed to kill PTY child process")
    }

    /// Get the shell name
    pub fn shell_name(&self) -> &str {
        &self.shell_name
    }

    /// Send a special key sequence
    pub fn send_key(&mut self, key: TerminalKey) -> Result<()> {
        let seq = key.to_escape_sequence();
        self.write(seq)
    }
}

/// Special terminal keys that need escape sequences
#[derive(Debug, Clone, Copy)]
pub enum TerminalKey {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Backspace,
    Tab,
    Enter,
    Escape,
    CtrlC,
    CtrlD,
    CtrlZ,
    CtrlL,
}

impl TerminalKey {
    /// Convert key to ANSI escape sequence
    pub fn to_escape_sequence(self) -> &'static [u8] {
        match self {
            TerminalKey::Up => b"\x1b[A",
            TerminalKey::Down => b"\x1b[B",
            TerminalKey::Right => b"\x1b[C",
            TerminalKey::Left => b"\x1b[D",
            TerminalKey::Home => b"\x1b[H",
            TerminalKey::End => b"\x1b[F",
            TerminalKey::PageUp => b"\x1b[5~",
            TerminalKey::PageDown => b"\x1b[6~",
            TerminalKey::Insert => b"\x1b[2~",
            TerminalKey::Delete => b"\x1b[3~",
            TerminalKey::Backspace => b"\x7f",
            TerminalKey::Tab => b"\t",
            TerminalKey::Enter => b"\r",
            TerminalKey::Escape => b"\x1b",
            TerminalKey::CtrlC => b"\x03",
            TerminalKey::CtrlD => b"\x04",
            TerminalKey::CtrlZ => b"\x1a",
            TerminalKey::CtrlL => b"\x0c",
        }
    }
}

impl Drop for PtyTerminal {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}
