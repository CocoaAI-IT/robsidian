//! Terminal UI panel

use eframe::egui::{self, Color32, FontId, Key, RichText};

use crate::terminal::{PtyTerminalState, TerminalKey, TerminalState};

/// Terminal panel
pub struct TerminalPanel;

impl TerminalPanel {
    /// Show the terminal panel
    pub fn show(ui: &mut egui::Ui, terminal: &mut TerminalState) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("Terminal");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear").clicked() {
                        terminal.clear_output();
                    }
                    if ui.button("+").on_hover_text("New terminal").clicked() {
                        terminal.new_tab();
                    }
                });
            });

            // Tab bar (if multiple terminals)
            if terminal.tabs.len() > 1 {
                ui.horizontal(|ui| {
                    for (idx, tab) in terminal.tabs.iter().enumerate() {
                        let label = format!("Terminal {}", idx + 1);
                        if ui.selectable_label(terminal.active_tab == idx, label).clicked() {
                            terminal.active_tab = idx;
                        }
                    }
                });
            }

            ui.separator();

            // Output area
            egui::ScrollArea::vertical()
                .id_salt("terminal_output")
                .stick_to_bottom(true)
                .max_height(ui.available_height() - 30.0)
                .show(ui, |ui| {
                    if let Some(tab) = terminal.tabs.get(terminal.active_tab) {
                        for line in &tab.output {
                            ui.monospace(line);
                        }
                    }
                });

            // Input area
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("$");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut terminal.input)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(ui.available_width() - 60.0),
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    terminal.execute_command();
                    response.request_focus();
                }

                if ui.button("Run").clicked() {
                    terminal.execute_command();
                }
            });
        });
    }
}

/// PTY Terminal panel for interactive shell sessions
pub struct PtyTerminalPanel;

impl PtyTerminalPanel {
    /// Show the PTY terminal panel
    pub fn show(ui: &mut egui::Ui, terminal: &mut PtyTerminalState, ctx: &egui::Context) {
        // Process any pending output
        terminal.process_all_output();

        ui.vertical(|ui| {
            // Header with shell info and controls
            ui.horizontal(|ui| {
                if let Some(tab) = terminal.current_tab() {
                    let shell_name = tab.pty.shell_name();
                    ui.heading(format!("Terminal ({})", shell_name));
                } else {
                    ui.heading("Terminal");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Shell selector dropdown
                    egui::ComboBox::from_id_salt("shell_selector")
                        .selected_text("+ New")
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(false, "Nushell").clicked() {
                                terminal.new_tab_with_shell("nu");
                            }
                            if ui.selectable_label(false, "PowerShell").clicked() {
                                terminal.new_tab_with_shell("pwsh");
                            }
                            #[cfg(windows)]
                            if ui.selectable_label(false, "CMD").clicked() {
                                terminal.new_tab_with_shell("cmd");
                            }
                            #[cfg(not(windows))]
                            if ui.selectable_label(false, "Bash").clicked() {
                                terminal.new_tab_with_shell("bash");
                            }
                        });

                    if terminal.tabs.len() > 1 {
                        if ui.button("Close").clicked() {
                            terminal.close_current_tab();
                        }
                    }
                });
            });

            // Tab bar (if multiple terminals)
            if terminal.tabs.len() > 1 {
                // Collect tab info first to avoid borrow issues
                let tab_info: Vec<(usize, String)> = terminal
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(idx, tab)| (idx, tab.pty.shell_name().to_string()))
                    .collect();

                let mut clicked_tab = None;
                ui.horizontal(|ui| {
                    for (idx, shell_name) in &tab_info {
                        let label = format!("{} {}", shell_name, idx + 1);
                        let selected = terminal.active_tab == *idx;

                        if ui.selectable_label(selected, label).clicked() {
                            clicked_tab = Some(*idx);
                        }
                    }
                });

                if let Some(idx) = clicked_tab {
                    terminal.switch_tab(idx);
                }
            }

            ui.separator();

            // Check for error state
            if let Some(tab) = terminal.current_tab() {
                if let Some(error) = &tab.error {
                    ui.colored_label(Color32::RED, error);
                    ui.add_space(8.0);
                    ui.label("Tips:");
                    ui.label("- Make sure Nushell is installed: https://www.nushell.sh/");
                    ui.label("- Or try a different shell from the dropdown");
                    return;
                }
            }

            // Terminal content area
            let available_rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());

            // Request focus when clicked
            if response.clicked() {
                response.request_focus();
            }

            // Handle keyboard input when focused
            if response.has_focus() {
                Self::handle_keyboard_input(ui, terminal);
            }

            // Draw terminal content
            Self::render_terminal_buffer(ui, terminal, available_rect);

            // Request continuous repainting for terminal updates
            ctx.request_repaint();
        });
    }

    /// Handle keyboard input for the PTY terminal
    fn handle_keyboard_input(ui: &mut egui::Ui, terminal: &mut PtyTerminalState) {
        let Some(tab) = terminal.current_tab_mut() else {
            return;
        };

        ui.input(|input| {
            // Handle special key combinations first
            let modifiers = input.modifiers;

            // Ctrl+C
            if modifiers.ctrl && input.key_pressed(Key::C) {
                let _ = tab.send_key(TerminalKey::CtrlC);
                return;
            }

            // Ctrl+D
            if modifiers.ctrl && input.key_pressed(Key::D) {
                let _ = tab.send_key(TerminalKey::CtrlD);
                return;
            }

            // Ctrl+Z
            if modifiers.ctrl && input.key_pressed(Key::Z) {
                let _ = tab.send_key(TerminalKey::CtrlZ);
                return;
            }

            // Ctrl+L (clear screen)
            if modifiers.ctrl && input.key_pressed(Key::L) {
                let _ = tab.send_key(TerminalKey::CtrlL);
                return;
            }

            // Arrow keys
            if input.key_pressed(Key::ArrowUp) {
                let _ = tab.send_key(TerminalKey::Up);
            }
            if input.key_pressed(Key::ArrowDown) {
                let _ = tab.send_key(TerminalKey::Down);
            }
            if input.key_pressed(Key::ArrowLeft) {
                let _ = tab.send_key(TerminalKey::Left);
            }
            if input.key_pressed(Key::ArrowRight) {
                let _ = tab.send_key(TerminalKey::Right);
            }

            // Home/End
            if input.key_pressed(Key::Home) {
                let _ = tab.send_key(TerminalKey::Home);
            }
            if input.key_pressed(Key::End) {
                let _ = tab.send_key(TerminalKey::End);
            }

            // Page Up/Down
            if input.key_pressed(Key::PageUp) {
                let _ = tab.send_key(TerminalKey::PageUp);
            }
            if input.key_pressed(Key::PageDown) {
                let _ = tab.send_key(TerminalKey::PageDown);
            }

            // Delete/Backspace
            if input.key_pressed(Key::Delete) {
                let _ = tab.send_key(TerminalKey::Delete);
            }
            if input.key_pressed(Key::Backspace) {
                let _ = tab.send_key(TerminalKey::Backspace);
            }

            // Tab
            if input.key_pressed(Key::Tab) {
                let _ = tab.send_key(TerminalKey::Tab);
            }

            // Enter
            if input.key_pressed(Key::Enter) {
                let _ = tab.send_key(TerminalKey::Enter);
            }

            // Escape
            if input.key_pressed(Key::Escape) {
                let _ = tab.send_key(TerminalKey::Escape);
            }

            // Regular text input
            for event in &input.events {
                if let egui::Event::Text(text) = event {
                    // Don't send if it was a ctrl combination
                    if !modifiers.ctrl {
                        let _ = tab.write(text.as_bytes());
                    }
                }
            }
        });
    }

    /// Render the terminal buffer content
    fn render_terminal_buffer(
        ui: &mut egui::Ui,
        terminal: &PtyTerminalState,
        rect: egui::Rect,
    ) {
        let Some(tab) = terminal.current_tab() else {
            return;
        };

        let painter = ui.painter_at(rect);
        let font_id = FontId::monospace(14.0);

        // Calculate character dimensions
        let char_width = 8.4; // Approximate for monospace
        let line_height = 16.0;

        let buffer = &tab.buffer;
        let cursor = buffer.cursor();

        // Draw background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 30));

        // Draw each line
        for (row_idx, line) in buffer.lines().iter().enumerate() {
            let y = rect.min.y + (row_idx as f32) * line_height;

            if y > rect.max.y {
                break; // Don't draw outside visible area
            }

            for (col_idx, styled_char) in line.chars.iter().enumerate() {
                let x = rect.min.x + (col_idx as f32) * char_width;

                if x > rect.max.x {
                    break;
                }

                let pos = egui::pos2(x, y);

                // Draw background if not transparent
                let bg = styled_char.effective_bg();
                if bg != Color32::TRANSPARENT {
                    let bg_rect = egui::Rect::from_min_size(
                        pos,
                        egui::vec2(char_width, line_height),
                    );
                    painter.rect_filled(bg_rect, 0.0, bg);
                }

                // Draw cursor
                if row_idx == cursor.row as usize && col_idx == cursor.col as usize {
                    let cursor_rect = egui::Rect::from_min_size(
                        pos,
                        egui::vec2(char_width, line_height),
                    );
                    painter.rect_filled(cursor_rect, 0.0, Color32::from_rgba_unmultiplied(255, 255, 255, 128));
                }

                // Draw character
                if styled_char.c != ' ' {
                    let fg = styled_char.effective_fg();
                    let mut text = RichText::new(styled_char.c.to_string())
                        .font(font_id.clone())
                        .color(fg);

                    if styled_char.bold {
                        text = text.strong();
                    }
                    if styled_char.italic {
                        text = text.italics();
                    }
                    if styled_char.underline {
                        text = text.underline();
                    }
                    if styled_char.strikethrough {
                        text = text.strikethrough();
                    }

                    painter.text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        styled_char.c.to_string(),
                        font_id.clone(),
                        fg,
                    );
                }
            }
        }
    }
}
