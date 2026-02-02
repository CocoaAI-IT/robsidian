//! Block rendering for live preview
//!
//! This module provides rendering functions for different markdown blocks,
//! used by the live preview editor to display formatted content.

use eframe::egui::{self, Color32, FontId, RichText, Ui};

use super::markdown_blocks::{InlineSpan, ListItem, ParsedBlock, TableCell};

/// Render a parsed block to the UI
pub fn render_block(ui: &mut Ui, block: &ParsedBlock) -> Option<BlockAction> {
    match block {
        ParsedBlock::Heading { level, text, .. } => render_heading(ui, *level, text),
        ParsedBlock::Paragraph { text, .. } => render_paragraph(ui, text),
        ParsedBlock::CodeBlock { lang, code, .. } => render_code_block(ui, lang.as_deref(), code),
        ParsedBlock::List {
            items,
            ordered,
            start,
            ..
        } => render_list(ui, items, *ordered, *start),
        ParsedBlock::WikiLink {
            target, display, ..
        } => render_wiki_link(ui, target, display.as_deref()),
        ParsedBlock::BlockQuote { content, .. } => render_blockquote(ui, content),
        ParsedBlock::HorizontalRule { .. } => {
            render_horizontal_rule(ui);
            None
        }
        ParsedBlock::Table { headers, rows, .. } => render_table(ui, headers, rows),
        ParsedBlock::Image {
            alt, url, title, ..
        } => render_image(ui, alt, url, title.as_deref()),
        ParsedBlock::BlankLine { .. } => {
            ui.add_space(8.0);
            None
        }
    }
}

/// Action that can be triggered by block interaction
#[derive(Debug, Clone)]
pub enum BlockAction {
    /// Navigate to a wiki link target
    NavigateToNote(String),
    /// Open external URL
    OpenUrl(String),
}

/// Render a heading
pub fn render_heading(ui: &mut Ui, level: u8, text: &str) -> Option<BlockAction> {
    let font_size = match level {
        1 => 28.0,
        2 => 24.0,
        3 => 20.0,
        4 => 18.0,
        5 => 16.0,
        _ => 14.0,
    };

    let text_color = match level {
        1 | 2 => Color32::from_rgb(200, 200, 200),
        _ => Color32::from_rgb(180, 180, 180),
    };

    ui.horizontal(|ui| {
        let rich_text = RichText::new(text)
            .font(FontId::proportional(font_size))
            .color(text_color)
            .strong();

        ui.label(rich_text);
    });

    // Add spacing after heading
    ui.add_space(match level {
        1 => 12.0,
        2 => 10.0,
        _ => 6.0,
    });

    None
}

/// Render a paragraph with inline formatting
pub fn render_paragraph(ui: &mut Ui, text: &str) -> Option<BlockAction> {
    let spans = super::markdown_blocks::parse_inline(text);
    let mut action = None;

    ui.horizontal_wrapped(|ui| {
        for span in &spans {
            match span {
                InlineSpan::Text(t) => {
                    ui.label(t);
                }
                InlineSpan::WikiLink { target, display } => {
                    let link_text = display.as_deref().unwrap_or(target);
                    let response = ui.link(link_text);
                    if response.clicked() {
                        action = Some(BlockAction::NavigateToNote(target.clone()));
                    }
                    if response.hovered() {
                        response.on_hover_text(format!("Open: {}", target));
                    }
                }
                InlineSpan::Code(code) => {
                    let text = RichText::new(code)
                        .font(FontId::monospace(14.0))
                        .background_color(Color32::from_rgb(45, 45, 45));
                    ui.label(text);
                }
                InlineSpan::Bold(t) => {
                    ui.label(RichText::new(t).strong());
                }
                InlineSpan::Italic(t) => {
                    ui.label(RichText::new(t).italics());
                }
                InlineSpan::Link { text, url } => {
                    let response = ui.link(text);
                    if response.clicked() {
                        action = Some(BlockAction::OpenUrl(url.clone()));
                    }
                }
            }
        }
    });

    ui.add_space(8.0);
    action
}

/// Render a code block with optional syntax highlighting
pub fn render_code_block(ui: &mut Ui, lang: Option<&str>, code: &str) -> Option<BlockAction> {
    let bg_color = Color32::from_rgb(40, 40, 40);
    let border_color = Color32::from_rgb(60, 60, 60);

    egui::Frame::none()
        .fill(bg_color)
        .stroke(egui::Stroke::new(1.0, border_color))
        .inner_margin(egui::Margin::same(8))
        .outer_margin(egui::Margin::symmetric(0, 4))
        .rounding(4.0)
        .show(ui, |ui| {
            // Language label
            if let Some(lang) = lang {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(lang)
                            .font(FontId::monospace(12.0))
                            .color(Color32::from_rgb(128, 128, 128)),
                    );
                });
                ui.add_space(4.0);
            }

            // Code content
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(code)
                        .font(FontId::monospace(14.0))
                        .color(Color32::from_rgb(200, 200, 200)),
                );
            });
        });

    ui.add_space(8.0);
    None
}

/// Render a list (ordered or unordered)
pub fn render_list(
    ui: &mut Ui,
    items: &[ListItem],
    ordered: bool,
    start: Option<u64>,
) -> Option<BlockAction> {
    let mut action = None;
    let start_num = start.unwrap_or(1);

    for (idx, item) in items.iter().enumerate() {
        ui.horizontal(|ui| {
            // Indent
            ui.add_space(16.0);

            // Bullet or number
            if let Some(checked) = item.checkbox {
                // Task list item
                let checkbox_text = if checked { "[x]" } else { "[ ]" };
                ui.label(
                    RichText::new(checkbox_text)
                        .font(FontId::monospace(14.0))
                        .color(Color32::from_rgb(150, 150, 150)),
                );
            } else if ordered {
                let num = start_num + idx as u64;
                ui.label(
                    RichText::new(format!("{}.", num))
                        .color(Color32::from_rgb(150, 150, 150)),
                );
            } else {
                ui.label(
                    RichText::new("•")
                        .color(Color32::from_rgb(150, 150, 150)),
                );
            }

            ui.add_space(4.0);

            // Item text (with inline parsing)
            let spans = super::markdown_blocks::parse_inline(&item.text);
            for span in &spans {
                match span {
                    InlineSpan::Text(t) => {
                        ui.label(t);
                    }
                    InlineSpan::WikiLink { target, display } => {
                        let link_text = display.as_deref().unwrap_or(target);
                        let response = ui.link(link_text);
                        if response.clicked() {
                            action = Some(BlockAction::NavigateToNote(target.clone()));
                        }
                    }
                    InlineSpan::Code(code) => {
                        let text = RichText::new(code)
                            .font(FontId::monospace(14.0))
                            .background_color(Color32::from_rgb(45, 45, 45));
                        ui.label(text);
                    }
                    _ => {}
                }
            }
        });

        // Render nested items (if any)
        if !item.children.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    if let Some(child_action) = render_list(ui, &item.children, ordered, None) {
                        action = Some(child_action);
                    }
                });
            });
        }
    }

    ui.add_space(8.0);
    action
}

/// Render a wiki link
pub fn render_wiki_link(ui: &mut Ui, target: &str, display: Option<&str>) -> Option<BlockAction> {
    let link_text = display.unwrap_or(target);
    let mut action = None;

    ui.horizontal(|ui| {
        let response = ui.link(
            RichText::new(link_text)
                .color(Color32::from_rgb(139, 180, 233))
                .underline(),
        );

        if response.clicked() {
            action = Some(BlockAction::NavigateToNote(target.to_string()));
        }

        if response.hovered() {
            response.on_hover_text(format!("Open note: {}", target));
        }
    });

    action
}

/// Render a blockquote
pub fn render_blockquote(ui: &mut Ui, content: &[ParsedBlock]) -> Option<BlockAction> {
    let mut action = None;

    egui::Frame::none()
        .fill(Color32::from_rgb(35, 35, 40))
        .inner_margin(egui::Margin {
            left: 12,
            right: 8,
            top: 8,
            bottom: 8,
        })
        .show(ui, |ui| {
            // Left border effect
            let rect = ui.max_rect();
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(4.0, rect.height()),
                ),
                0.0,
                Color32::from_rgb(100, 100, 120),
            );

            ui.add_space(8.0);

            for block in content {
                if let Some(a) = render_block(ui, block) {
                    action = Some(a);
                }
            }
        });

    ui.add_space(8.0);
    action
}

/// Render a horizontal rule
pub fn render_horizontal_rule(ui: &mut Ui) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
}

/// Render a table
pub fn render_table(
    ui: &mut Ui,
    headers: &[TableCell],
    rows: &[Vec<TableCell>],
) -> Option<BlockAction> {
    use egui_extras::{Column, TableBuilder};

    let num_cols = headers.len().max(1);

    TableBuilder::new(ui)
        .striped(true)
        .columns(Column::auto().at_least(60.0), num_cols)
        .header(20.0, |mut header| {
            for cell in headers {
                header.col(|ui| {
                    ui.strong(&cell.content);
                });
            }
        })
        .body(|mut body| {
            for row in rows {
                body.row(18.0, |mut row_ui| {
                    for cell in row {
                        row_ui.col(|ui| {
                            ui.label(&cell.content);
                        });
                    }
                });
            }
        });

    ui.add_space(8.0);
    None
}

/// Render an image (placeholder for now)
pub fn render_image(
    ui: &mut Ui,
    alt: &str,
    url: &str,
    _title: Option<&str>,
) -> Option<BlockAction> {
    // For now, just show a placeholder with the alt text and URL
    // Full image loading would require async loading and caching

    egui::Frame::none()
        .fill(Color32::from_rgb(45, 45, 50))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(70, 70, 70)))
        .inner_margin(egui::Margin::same(8))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📷")
                        .font(FontId::proportional(24.0)),
                );
                ui.vertical(|ui| {
                    if !alt.is_empty() {
                        ui.label(RichText::new(alt).italics());
                    }
                    ui.label(
                        RichText::new(url)
                            .font(FontId::monospace(12.0))
                            .color(Color32::from_rgb(128, 128, 128)),
                    );
                });
            });
        });

    ui.add_space(8.0);
    None
}

/// Render raw markdown text (for editing mode)
pub fn render_raw_block(ui: &mut Ui, content: &str) -> egui::Response {
    let mut text = content.to_string();
    let text_edit = egui::TextEdit::multiline(&mut text)
        .font(FontId::monospace(14.0))
        .desired_width(ui.available_width())
        .frame(false);

    ui.add(text_edit)
}
