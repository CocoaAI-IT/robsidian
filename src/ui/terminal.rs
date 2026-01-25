//! Terminal UI panel

use eframe::egui;

use crate::terminal::TerminalState;

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
