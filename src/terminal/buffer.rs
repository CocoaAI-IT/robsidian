//! Terminal buffer management
//!
//! This module provides a terminal buffer that stores styled characters,
//! manages cursor position, and handles scrolling.

use egui::Color32;

/// A single styled character in the terminal
#[derive(Debug, Clone, Copy)]
pub struct StyledChar {
    pub c: char,
    pub fg: Color32,
    pub bg: Color32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub inverse: bool,
}

impl Default for StyledChar {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color32::LIGHT_GRAY,
            bg: Color32::TRANSPARENT,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            inverse: false,
        }
    }
}

impl StyledChar {
    /// Create a new styled character with default styling
    pub fn new(c: char) -> Self {
        Self {
            c,
            ..Default::default()
        }
    }

    /// Get the effective foreground color (considering inverse)
    pub fn effective_fg(&self) -> Color32 {
        if self.inverse {
            self.bg
        } else {
            self.fg
        }
    }

    /// Get the effective background color (considering inverse)
    pub fn effective_bg(&self) -> Color32 {
        if self.inverse {
            self.fg
        } else {
            self.bg
        }
    }
}

/// A single line in the terminal buffer
#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub chars: Vec<StyledChar>,
    pub wrapped: bool, // Whether this line is a continuation of the previous
}

impl TerminalLine {
    /// Create a new empty line with the given width
    pub fn new(width: usize) -> Self {
        Self {
            chars: vec![StyledChar::default(); width],
            wrapped: false,
        }
    }

    /// Get the character at the given column
    pub fn get(&self, col: usize) -> Option<&StyledChar> {
        self.chars.get(col)
    }

    /// Set the character at the given column
    pub fn set(&mut self, col: usize, ch: StyledChar) {
        if col < self.chars.len() {
            self.chars[col] = ch;
        }
    }

    /// Clear the line with spaces
    pub fn clear(&mut self) {
        for ch in &mut self.chars {
            *ch = StyledChar::default();
        }
        self.wrapped = false;
    }

    /// Resize the line to the given width
    pub fn resize(&mut self, width: usize) {
        self.chars.resize(width, StyledChar::default());
    }

    /// Get the content as a string (trimming trailing spaces)
    pub fn to_string_trimmed(&self) -> String {
        let s: String = self.chars.iter().map(|c| c.c).collect();
        s.trim_end().to_string()
    }
}

/// Cursor position in the terminal
#[derive(Debug, Clone, Copy, Default)]
pub struct CursorPos {
    pub row: u16,
    pub col: u16,
}

/// Terminal buffer that stores the screen content
pub struct TerminalBuffer {
    lines: Vec<TerminalLine>,
    scrollback: Vec<TerminalLine>,
    cursor: CursorPos,
    saved_cursor: Option<CursorPos>,
    scroll_region: (u16, u16), // (top, bottom) of scroll region
    size: (u16, u16),          // (cols, rows)
    current_style: StyledChar, // Current style for new characters
    max_scrollback: usize,
}

impl TerminalBuffer {
    /// Create a new terminal buffer with the given size
    pub fn new(cols: u16, rows: u16) -> Self {
        let lines = (0..rows)
            .map(|_| TerminalLine::new(cols as usize))
            .collect();

        Self {
            lines,
            scrollback: Vec::new(),
            cursor: CursorPos::default(),
            saved_cursor: None,
            scroll_region: (0, rows.saturating_sub(1)),
            size: (cols, rows),
            current_style: StyledChar::default(),
            max_scrollback: 10000,
        }
    }

    /// Get the buffer size (cols, rows)
    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    /// Get the cursor position
    pub fn cursor(&self) -> CursorPos {
        self.cursor
    }

    /// Set the cursor position (clamped to buffer bounds)
    pub fn set_cursor(&mut self, col: u16, row: u16) {
        self.cursor.col = col.min(self.size.0.saturating_sub(1));
        self.cursor.row = row.min(self.size.1.saturating_sub(1));
    }

    /// Move cursor relatively
    pub fn move_cursor(&mut self, dcol: i16, drow: i16) {
        let new_col = (self.cursor.col as i16 + dcol).max(0) as u16;
        let new_row = (self.cursor.row as i16 + drow).max(0) as u16;
        self.set_cursor(new_col, new_row);
    }

    /// Save cursor position
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor);
    }

    /// Restore cursor position
    pub fn restore_cursor(&mut self) {
        if let Some(pos) = self.saved_cursor {
            self.cursor = pos;
        }
    }

    /// Get a line by row index
    pub fn line(&self, row: usize) -> Option<&TerminalLine> {
        self.lines.get(row)
    }

    /// Get all visible lines
    pub fn lines(&self) -> &[TerminalLine] {
        &self.lines
    }

    /// Get scrollback lines
    pub fn scrollback(&self) -> &[TerminalLine] {
        &self.scrollback
    }

    /// Put a character at the current cursor position
    pub fn put_char(&mut self, c: char) {
        if self.cursor.col >= self.size.0 {
            // Wrap to next line
            self.cursor.col = 0;
            self.cursor.row += 1;
            if self.cursor.row >= self.size.1 {
                self.scroll_up(1);
                self.cursor.row = self.size.1 - 1;
            }
        }

        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;

        if row < self.lines.len() {
            let mut styled = self.current_style;
            styled.c = c;
            self.lines[row].set(col, styled);
        }

        self.cursor.col += 1;
    }

    /// Handle newline
    pub fn newline(&mut self) {
        self.cursor.col = 0;
        self.cursor.row += 1;
        if self.cursor.row > self.scroll_region.1 {
            self.scroll_up(1);
            self.cursor.row = self.scroll_region.1;
        }
    }

    /// Handle carriage return
    pub fn carriage_return(&mut self) {
        self.cursor.col = 0;
    }

    /// Handle backspace
    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    /// Handle tab
    pub fn tab(&mut self) {
        let tab_stop = 8;
        let next_tab = ((self.cursor.col / tab_stop) + 1) * tab_stop;
        self.cursor.col = next_tab.min(self.size.0 - 1);
    }

    /// Scroll up by n lines within the scroll region
    pub fn scroll_up(&mut self, n: u16) {
        let top = self.scroll_region.0 as usize;
        let bottom = self.scroll_region.1 as usize;

        for _ in 0..n {
            if top < self.lines.len() {
                // Move top line to scrollback
                let line = self.lines.remove(top);
                self.scrollback.push(line);

                // Trim scrollback if needed
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.remove(0);
                }

                // Insert new line at bottom
                let new_line = TerminalLine::new(self.size.0 as usize);
                let insert_pos = bottom.min(self.lines.len());
                self.lines.insert(insert_pos, new_line);
            }
        }
    }

    /// Scroll down by n lines within the scroll region
    pub fn scroll_down(&mut self, n: u16) {
        let top = self.scroll_region.0 as usize;
        let bottom = self.scroll_region.1 as usize;

        for _ in 0..n {
            if bottom < self.lines.len() {
                self.lines.remove(bottom);
                let new_line = TerminalLine::new(self.size.0 as usize);
                self.lines.insert(top, new_line);
            }
        }
    }

    /// Set the scroll region
    pub fn set_scroll_region(&mut self, top: u16, bottom: u16) {
        self.scroll_region = (
            top.min(self.size.1.saturating_sub(1)),
            bottom.min(self.size.1.saturating_sub(1)),
        );
    }

    /// Reset scroll region to full screen
    pub fn reset_scroll_region(&mut self) {
        self.scroll_region = (0, self.size.1.saturating_sub(1));
    }

    /// Clear from cursor to end of line
    pub fn clear_to_eol(&mut self) {
        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;
        if row < self.lines.len() {
            for i in col..self.size.0 as usize {
                self.lines[row].set(i, StyledChar::default());
            }
        }
    }

    /// Clear from cursor to beginning of line
    pub fn clear_to_bol(&mut self) {
        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;
        if row < self.lines.len() {
            for i in 0..=col {
                self.lines[row].set(i, StyledChar::default());
            }
        }
    }

    /// Clear entire line
    pub fn clear_line(&mut self) {
        let row = self.cursor.row as usize;
        if row < self.lines.len() {
            self.lines[row].clear();
        }
    }

    /// Clear from cursor to end of screen
    pub fn clear_to_eos(&mut self) {
        self.clear_to_eol();
        for row in (self.cursor.row as usize + 1)..self.lines.len() {
            self.lines[row].clear();
        }
    }

    /// Clear from cursor to beginning of screen
    pub fn clear_to_bos(&mut self) {
        self.clear_to_bol();
        for row in 0..self.cursor.row as usize {
            self.lines[row].clear();
        }
    }

    /// Clear entire screen
    pub fn clear_screen(&mut self) {
        for line in &mut self.lines {
            line.clear();
        }
    }

    /// Set current text style
    pub fn set_style(&mut self, style: StyledChar) {
        self.current_style = style;
    }

    /// Get current text style
    pub fn current_style(&self) -> &StyledChar {
        &self.current_style
    }

    /// Get mutable current text style
    pub fn current_style_mut(&mut self) -> &mut StyledChar {
        &mut self.current_style
    }

    /// Reset current style to default
    pub fn reset_style(&mut self) {
        self.current_style = StyledChar::default();
    }

    /// Resize the buffer
    pub fn resize(&mut self, cols: u16, rows: u16) {
        // Resize existing lines
        for line in &mut self.lines {
            line.resize(cols as usize);
        }

        // Add or remove lines as needed
        while self.lines.len() < rows as usize {
            self.lines.push(TerminalLine::new(cols as usize));
        }
        while self.lines.len() > rows as usize {
            self.lines.pop();
        }

        self.size = (cols, rows);
        self.scroll_region = (0, rows.saturating_sub(1));

        // Clamp cursor
        self.cursor.col = self.cursor.col.min(cols.saturating_sub(1));
        self.cursor.row = self.cursor.row.min(rows.saturating_sub(1));
    }

    /// Erase n characters from cursor position
    pub fn erase_chars(&mut self, n: u16) {
        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;
        if row < self.lines.len() {
            for i in col..(col + n as usize).min(self.size.0 as usize) {
                self.lines[row].set(i, StyledChar::default());
            }
        }
    }

    /// Insert n blank lines at cursor row
    pub fn insert_lines(&mut self, n: u16) {
        let row = self.cursor.row as usize;
        let bottom = self.scroll_region.1 as usize;

        for _ in 0..n {
            if bottom < self.lines.len() && row <= bottom {
                self.lines.remove(bottom);
                self.lines.insert(row, TerminalLine::new(self.size.0 as usize));
            }
        }
    }

    /// Delete n lines at cursor row
    pub fn delete_lines(&mut self, n: u16) {
        let row = self.cursor.row as usize;
        let bottom = self.scroll_region.1 as usize;

        for _ in 0..n {
            if row < self.lines.len() && row <= bottom {
                self.lines.remove(row);
                self.lines.insert(bottom, TerminalLine::new(self.size.0 as usize));
            }
        }
    }
}

/// Standard ANSI colors
pub const ANSI_COLORS: [Color32; 16] = [
    Color32::from_rgb(0, 0, 0),       // Black
    Color32::from_rgb(205, 49, 49),   // Red
    Color32::from_rgb(13, 188, 121),  // Green
    Color32::from_rgb(229, 229, 16),  // Yellow
    Color32::from_rgb(36, 114, 200),  // Blue
    Color32::from_rgb(188, 63, 188),  // Magenta
    Color32::from_rgb(17, 168, 205),  // Cyan
    Color32::from_rgb(229, 229, 229), // White
    // Bright colors
    Color32::from_rgb(102, 102, 102), // Bright Black (Gray)
    Color32::from_rgb(241, 76, 76),   // Bright Red
    Color32::from_rgb(35, 209, 139),  // Bright Green
    Color32::from_rgb(245, 245, 67),  // Bright Yellow
    Color32::from_rgb(59, 142, 234),  // Bright Blue
    Color32::from_rgb(214, 112, 214), // Bright Magenta
    Color32::from_rgb(41, 184, 219),  // Bright Cyan
    Color32::from_rgb(255, 255, 255), // Bright White
];

/// Convert 256-color index to Color32
pub fn color_256_to_rgb(index: u8) -> Color32 {
    if index < 16 {
        ANSI_COLORS[index as usize]
    } else if index < 232 {
        // 6x6x6 color cube
        let index = index - 16;
        let r = (index / 36) % 6;
        let g = (index / 6) % 6;
        let b = index % 6;
        let to_rgb = |c: u8| if c == 0 { 0 } else { 55 + c * 40 };
        Color32::from_rgb(to_rgb(r), to_rgb(g), to_rgb(b))
    } else {
        // Grayscale
        let gray = 8 + (index - 232) * 10;
        Color32::from_rgb(gray, gray, gray)
    }
}
