//! Live preview editor for Obsidian-style editing
//!
//! This module provides a hybrid editor that shows:
//! - Raw markdown for the block containing the cursor (editable)
//! - Rendered preview for all other blocks
//!
//! This creates an Obsidian-like editing experience where you can see
//! formatted output while still being able to edit.

use eframe::egui::{self, Color32, FontId, ScrollArea, TextEdit, Ui};

use super::block_renderer::{render_block, BlockAction};
use super::markdown_blocks::{find_block_at_position, parse_blocks, ParsedBlock};
use crate::core::document::Document;

/// Live preview editor state
pub struct LivePreviewEditor {
    /// Current cursor byte position in the document
    cursor_byte_pos: usize,
    /// Cached parsed blocks
    parsed_blocks: Vec<ParsedBlock>,
    /// Cache of the content that was parsed (to detect changes)
    cached_content: String,
    /// Index of the block being edited (if any)
    editing_block: Option<usize>,
}

impl Default for LivePreviewEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl LivePreviewEditor {
    /// Create a new live preview editor
    pub fn new() -> Self {
        Self {
            cursor_byte_pos: 0,
            parsed_blocks: Vec::new(),
            cached_content: String::new(),
            editing_block: None,
        }
    }

    /// Update the editor with document content
    fn update_blocks(&mut self, content: &str) {
        if content != self.cached_content {
            self.parsed_blocks = parse_blocks(content);
            self.cached_content = content.to_string();
        }
    }

    /// Find which block contains the cursor
    fn find_cursor_block(&self) -> Option<usize> {
        find_block_at_position(&self.parsed_blocks, self.cursor_byte_pos)
    }

    /// Show the live preview editor
    pub fn show(
        &mut self,
        ui: &mut Ui,
        document: &mut Document,
    ) -> Option<BlockAction> {
        let content = document.content.clone();
        self.update_blocks(&content);

        let mut action = None;
        let mut new_content = content.clone();
        let mut content_changed = false;

        ScrollArea::vertical()
            .id_salt("live_preview_scroll")
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                for (idx, block) in self.parsed_blocks.iter().enumerate() {
                    let is_editing = self.editing_block == Some(idx);
                    let block_range = block.range();

                    // Create a frame for the block
                    ui.push_id(idx, |ui| {
                        // Make the entire block area interactive
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 0.0),
                            egui::Sense::click(),
                        );

                        if is_editing {
                            // Show raw markdown for editing
                            let block_content = &content[block_range.clone()];
                            let mut edit_text = block_content.to_string();

                            let text_response = ui.add(
                                TextEdit::multiline(&mut edit_text)
                                    .font(FontId::monospace(14.0))
                                    .desired_width(ui.available_width())
                                    .frame(true)
                                    .margin(egui::Margin::same(4)),
                            );

                            // Update content if changed
                            if edit_text != block_content {
                                new_content = format!(
                                    "{}{}{}",
                                    &content[..block_range.start],
                                    edit_text,
                                    &content[block_range.end..]
                                );
                                content_changed = true;
                            }

                            // Click outside to exit edit mode
                            if text_response.clicked_elsewhere() {
                                self.editing_block = None;
                            }
                        } else {
                            // Show rendered preview
                            egui::Frame::new()
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    if let Some(a) = render_block(ui, block) {
                                        action = Some(a);
                                    }
                                });

                            // Click to start editing this block
                            if response.clicked() {
                                self.editing_block = Some(idx);
                                self.cursor_byte_pos = block_range.start;
                            }

                            // Hover effect
                            if response.hovered() {
                                ui.painter().rect_stroke(
                                    rect,
                                    egui::CornerRadius::same(4),
                                    egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 50)),
                                    egui::StrokeKind::Outside,
                                );
                            }
                        }
                    });
                }

                // Add some space at the bottom for clicking to add content
                let (rect, add_response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 50.0),
                    egui::Sense::click(),
                );

                if add_response.clicked() {
                    // Start editing at the end
                    self.editing_block = Some(self.parsed_blocks.len());
                    self.cursor_byte_pos = content.len();
                }

                if add_response.hovered() {
                    ui.painter().rect_stroke(
                        rect,
                        egui::CornerRadius::same(4),
                        egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 30)),
                        egui::StrokeKind::Outside,
                    );
                }
            });

        // Apply content changes
        if content_changed {
            document.set_content(new_content);
            // Re-parse after change
            self.update_blocks(&document.content);
        }

        action
    }
}

/// Simplified live preview that shows the whole document
/// with formatting, suitable for read-only preview or simpler editing
pub struct SimpleLivePreview;

impl SimpleLivePreview {
    /// Show a simplified live preview (read-only)
    pub fn show(ui: &mut Ui, content: &str) -> Option<BlockAction> {
        let blocks = parse_blocks(content);
        let mut action = None;

        ScrollArea::vertical()
            .id_salt("simple_live_preview")
            .show(ui, |ui| {
                for block in &blocks {
                    if let Some(a) = render_block(ui, block) {
                        action = Some(a);
                    }
                }
            });

        action
    }
}
