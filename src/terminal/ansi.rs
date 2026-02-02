//! ANSI escape sequence parser using vte
//!
//! This module parses ANSI escape sequences from terminal output
//! and applies them to a TerminalBuffer.

use crate::terminal::buffer::{color_256_to_rgb, TerminalBuffer, ANSI_COLORS};
use egui::Color32;
use vte::{Params, Perform};

/// ANSI parser that processes terminal output
pub struct AnsiParser {
    parser: vte::Parser,
}

impl AnsiParser {
    /// Create a new ANSI parser
    pub fn new() -> Self {
        Self {
            parser: vte::Parser::new(),
        }
    }

    /// Process input data and update the buffer
    pub fn process(&mut self, data: &[u8], buffer: &mut TerminalBuffer) {
        let mut performer = TerminalPerformer { buffer };
        for byte in data {
            self.parser.advance(&mut performer, *byte);
        }
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Performer that applies ANSI sequences to a TerminalBuffer
struct TerminalPerformer<'a> {
    buffer: &'a mut TerminalBuffer,
}

impl<'a> Perform for TerminalPerformer<'a> {
    fn print(&mut self, c: char) {
        self.buffer.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x07 => {} // Bell - ignore
            0x08 => self.buffer.backspace(),
            0x09 => self.buffer.tab(),
            0x0A => self.buffer.newline(),
            0x0D => self.buffer.carriage_return(),
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences - not commonly needed
    }

    fn put(&mut self, _byte: u8) {
        // DCS data - not commonly needed
    }

    fn unhook(&mut self) {
        // End DCS sequence
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences (e.g., window title)
        if params.is_empty() {
            return;
        }

        // Handle OSC 0, 1, 2 (window title) - we ignore these for now
        // Handle OSC 8 (hyperlinks) - we ignore these for now
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let params: Vec<u16> = params.iter().map(|p| p.first().copied().unwrap_or(0)).collect();

        match action {
            // Cursor movement
            'A' => {
                // Cursor Up
                let n = params.first().copied().unwrap_or(1).max(1) as i16;
                self.buffer.move_cursor(0, -n);
            }
            'B' => {
                // Cursor Down
                let n = params.first().copied().unwrap_or(1).max(1) as i16;
                self.buffer.move_cursor(0, n);
            }
            'C' => {
                // Cursor Forward
                let n = params.first().copied().unwrap_or(1).max(1) as i16;
                self.buffer.move_cursor(n, 0);
            }
            'D' => {
                // Cursor Back
                let n = params.first().copied().unwrap_or(1).max(1) as i16;
                self.buffer.move_cursor(-n, 0);
            }
            'E' => {
                // Cursor Next Line
                let n = params.first().copied().unwrap_or(1).max(1) as i16;
                self.buffer.move_cursor(0, n);
                self.buffer.carriage_return();
            }
            'F' => {
                // Cursor Previous Line
                let n = params.first().copied().unwrap_or(1).max(1) as i16;
                self.buffer.move_cursor(0, -n);
                self.buffer.carriage_return();
            }
            'G' => {
                // Cursor Horizontal Absolute
                let col = params.first().copied().unwrap_or(1).saturating_sub(1);
                let row = self.buffer.cursor().row;
                self.buffer.set_cursor(col, row);
            }
            'H' | 'f' => {
                // Cursor Position
                let row = params.first().copied().unwrap_or(1).saturating_sub(1);
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1);
                self.buffer.set_cursor(col, row);
            }

            // Erase
            'J' => {
                // Erase in Display
                match params.first().copied().unwrap_or(0) {
                    0 => self.buffer.clear_to_eos(),
                    1 => self.buffer.clear_to_bos(),
                    2 | 3 => self.buffer.clear_screen(),
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line
                match params.first().copied().unwrap_or(0) {
                    0 => self.buffer.clear_to_eol(),
                    1 => self.buffer.clear_to_bol(),
                    2 => self.buffer.clear_line(),
                    _ => {}
                }
            }

            // Insert/Delete
            'L' => {
                // Insert Lines
                let n = params.first().copied().unwrap_or(1).max(1);
                self.buffer.insert_lines(n);
            }
            'M' => {
                // Delete Lines
                let n = params.first().copied().unwrap_or(1).max(1);
                self.buffer.delete_lines(n);
            }
            'X' => {
                // Erase Characters
                let n = params.first().copied().unwrap_or(1).max(1);
                self.buffer.erase_chars(n);
            }

            // Scroll
            'S' => {
                // Scroll Up
                let n = params.first().copied().unwrap_or(1).max(1);
                self.buffer.scroll_up(n);
            }
            'T' => {
                // Scroll Down
                let n = params.first().copied().unwrap_or(1).max(1);
                self.buffer.scroll_down(n);
            }

            // SGR - Select Graphic Rendition
            'm' => {
                self.handle_sgr(&params);
            }

            // Cursor save/restore
            's' => self.buffer.save_cursor(),
            'u' => self.buffer.restore_cursor(),

            // Scroll region
            'r' => {
                // Set Scrolling Region
                let top = params.first().copied().unwrap_or(1).saturating_sub(1);
                let (_, rows) = self.buffer.size();
                let bottom = params.get(1).copied().unwrap_or(rows).saturating_sub(1);
                self.buffer.set_scroll_region(top, bottom);
            }

            // Private modes (DECSET/DECRST) - common ones
            'h' | 'l' => {
                // Set/Reset mode - we handle a few common ones
                // Most are ignored for simplicity
            }

            _ => {
                // Unknown CSI sequence
                tracing::trace!("Unknown CSI sequence: {} with params {:?}", action, params);
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'7' => self.buffer.save_cursor(),    // DECSC
            b'8' => self.buffer.restore_cursor(), // DECRC
            b'D' => self.buffer.newline(),        // IND - Index
            b'E' => {
                // NEL - Next Line
                self.buffer.newline();
                self.buffer.carriage_return();
            }
            b'M' => {
                // RI - Reverse Index
                let cursor = self.buffer.cursor();
                if cursor.row == 0 {
                    self.buffer.scroll_down(1);
                } else {
                    self.buffer.move_cursor(0, -1);
                }
            }
            b'c' => {
                // RIS - Reset
                self.buffer.clear_screen();
                self.buffer.set_cursor(0, 0);
                self.buffer.reset_style();
                self.buffer.reset_scroll_region();
            }
            _ => {}
        }
    }
}

impl<'a> TerminalPerformer<'a> {
    /// Handle SGR (Select Graphic Rendition) sequences
    fn handle_sgr(&mut self, params: &[u16]) {
        let mut i = 0;
        let params = if params.is_empty() { &[0u16][..] } else { params };

        while i < params.len() {
            let param = params[i];
            match param {
                0 => {
                    // Reset
                    self.buffer.reset_style();
                }
                1 => {
                    // Bold
                    self.buffer.current_style_mut().bold = true;
                }
                2 => {
                    // Dim - we treat as non-bold
                    self.buffer.current_style_mut().bold = false;
                }
                3 => {
                    // Italic
                    self.buffer.current_style_mut().italic = true;
                }
                4 => {
                    // Underline
                    self.buffer.current_style_mut().underline = true;
                }
                7 => {
                    // Inverse
                    self.buffer.current_style_mut().inverse = true;
                }
                9 => {
                    // Strikethrough
                    self.buffer.current_style_mut().strikethrough = true;
                }
                22 => {
                    // Normal intensity (not bold, not dim)
                    self.buffer.current_style_mut().bold = false;
                }
                23 => {
                    // Not italic
                    self.buffer.current_style_mut().italic = false;
                }
                24 => {
                    // Not underlined
                    self.buffer.current_style_mut().underline = false;
                }
                27 => {
                    // Not inverse
                    self.buffer.current_style_mut().inverse = false;
                }
                29 => {
                    // Not strikethrough
                    self.buffer.current_style_mut().strikethrough = false;
                }

                // Foreground colors
                30..=37 => {
                    let color_idx = (param - 30) as usize;
                    self.buffer.current_style_mut().fg = ANSI_COLORS[color_idx];
                }
                38 => {
                    // Extended foreground color
                    if let Some(color) = self.parse_extended_color(params, &mut i) {
                        self.buffer.current_style_mut().fg = color;
                    }
                }
                39 => {
                    // Default foreground
                    self.buffer.current_style_mut().fg = Color32::LIGHT_GRAY;
                }

                // Background colors
                40..=47 => {
                    let color_idx = (param - 40) as usize;
                    self.buffer.current_style_mut().bg = ANSI_COLORS[color_idx];
                }
                48 => {
                    // Extended background color
                    if let Some(color) = self.parse_extended_color(params, &mut i) {
                        self.buffer.current_style_mut().bg = color;
                    }
                }
                49 => {
                    // Default background
                    self.buffer.current_style_mut().bg = Color32::TRANSPARENT;
                }

                // Bright foreground colors
                90..=97 => {
                    let color_idx = (param - 90 + 8) as usize;
                    self.buffer.current_style_mut().fg = ANSI_COLORS[color_idx];
                }

                // Bright background colors
                100..=107 => {
                    let color_idx = (param - 100 + 8) as usize;
                    self.buffer.current_style_mut().bg = ANSI_COLORS[color_idx];
                }

                _ => {
                    // Unknown SGR parameter
                }
            }
            i += 1;
        }
    }

    /// Parse extended color (256-color or RGB)
    fn parse_extended_color(&self, params: &[u16], i: &mut usize) -> Option<Color32> {
        if *i + 1 >= params.len() {
            return None;
        }

        let mode = params[*i + 1];
        match mode {
            5 => {
                // 256-color mode
                if *i + 2 < params.len() {
                    *i += 2;
                    Some(color_256_to_rgb(params[*i] as u8))
                } else {
                    None
                }
            }
            2 => {
                // RGB mode
                if *i + 4 < params.len() {
                    let r = params[*i + 2] as u8;
                    let g = params[*i + 3] as u8;
                    let b = params[*i + 4] as u8;
                    *i += 4;
                    Some(Color32::from_rgb(r, g, b))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
