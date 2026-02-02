//! Markdown block parsing for live preview
//!
//! This module parses markdown content into discrete blocks with byte ranges,
//! enabling cursor-aware rendering where the block containing the cursor
//! shows raw markdown while others show rendered output.

use std::ops::Range;

/// A list item with potential checkbox state
#[derive(Debug, Clone)]
pub struct ListItem {
    /// The text content of the list item
    pub text: String,
    /// Checkbox state: Some(true) = checked, Some(false) = unchecked, None = no checkbox
    pub checkbox: Option<bool>,
    /// Nested items (for sublists)
    pub children: Vec<ListItem>,
}

/// A table cell
#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: String,
    pub alignment: TableAlignment,
}

/// Table column alignment
#[derive(Debug, Clone, Copy, Default)]
pub enum TableAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// A parsed markdown block with its byte range in the source
#[derive(Debug, Clone)]
pub enum ParsedBlock {
    /// Heading with level (1-6) and text
    Heading {
        level: u8,
        text: String,
        range: Range<usize>,
    },

    /// Regular paragraph
    Paragraph {
        text: String,
        range: Range<usize>,
    },

    /// Fenced code block
    CodeBlock {
        lang: Option<String>,
        code: String,
        range: Range<usize>,
    },

    /// Unordered or ordered list
    List {
        items: Vec<ListItem>,
        ordered: bool,
        start: Option<u64>,
        range: Range<usize>,
    },

    /// Wiki-style link [[target]] or [[target|display]]
    WikiLink {
        target: String,
        display: Option<String>,
        range: Range<usize>,
    },

    /// Block quote
    BlockQuote {
        content: Vec<ParsedBlock>,
        range: Range<usize>,
    },

    /// Horizontal rule
    HorizontalRule {
        range: Range<usize>,
    },

    /// Table
    Table {
        headers: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
        range: Range<usize>,
    },

    /// Image
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        range: Range<usize>,
    },

    /// Blank line(s)
    BlankLine {
        range: Range<usize>,
    },
}

impl ParsedBlock {
    /// Get the byte range of this block in the source
    pub fn range(&self) -> Range<usize> {
        match self {
            ParsedBlock::Heading { range, .. } => range.clone(),
            ParsedBlock::Paragraph { range, .. } => range.clone(),
            ParsedBlock::CodeBlock { range, .. } => range.clone(),
            ParsedBlock::List { range, .. } => range.clone(),
            ParsedBlock::WikiLink { range, .. } => range.clone(),
            ParsedBlock::BlockQuote { range, .. } => range.clone(),
            ParsedBlock::HorizontalRule { range, .. } => range.clone(),
            ParsedBlock::Table { range, .. } => range.clone(),
            ParsedBlock::Image { range, .. } => range.clone(),
            ParsedBlock::BlankLine { range, .. } => range.clone(),
        }
    }

    /// Check if the given byte position is within this block
    pub fn contains(&self, byte_pos: usize) -> bool {
        let range = self.range();
        byte_pos >= range.start && byte_pos < range.end
    }
}

/// Parse markdown content into blocks
pub fn parse_blocks(content: &str) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let mut current_pos = 0;

    // First, extract wiki links and replace with placeholders
    // This is done before pulldown-cmark parsing since it doesn't understand wiki links
    let (processed_content, _wiki_links) = extract_wiki_links(content);

    // Use pulldown-cmark for standard markdown parsing
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&processed_content, options);

    let mut block_start = 0;
    let mut current_text = String::new();
    let mut code_lang: Option<String> = None;
    let mut list_items: Vec<ListItem> = Vec::new();
    let mut list_ordered = false;
    let mut list_start: Option<u64> = None;
    let mut in_list_item = false;
    let mut current_item_text = String::new();
    let mut item_checkbox: Option<bool> = None;
    let mut table_headers: Vec<TableCell> = Vec::new();
    let mut table_rows: Vec<Vec<TableCell>> = Vec::new();
    let mut current_row: Vec<TableCell> = Vec::new();
    let mut in_table_head = false;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(tag) => {
                match &tag {
                    Tag::Heading { level: _, .. } => {
                        block_start = range.start;
                        current_text.clear();
                    }
                    Tag::Paragraph => {
                        block_start = range.start;
                        current_text.clear();
                    }
                    Tag::CodeBlock(kind) => {
                        block_start = range.start;
                        current_text.clear();
                        code_lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                                let lang = lang.to_string();
                                if lang.is_empty() {
                                    None
                                } else {
                                    Some(lang)
                                }
                            }
                            _ => None,
                        };
                    }
                    Tag::List(start) => {
                        block_start = range.start;
                        list_ordered = start.is_some();
                        list_start = *start;
                        list_items.clear();
                    }
                    Tag::Item => {
                        in_list_item = true;
                        current_item_text.clear();
                        item_checkbox = None;
                    }
                    Tag::BlockQuote(_) => {
                        block_start = range.start;
                        current_text.clear();
                    }
                    Tag::Table(_) => {
                        block_start = range.start;
                        table_headers.clear();
                        table_rows.clear();
                    }
                    Tag::TableHead => {
                        in_table_head = true;
                        current_row.clear();
                    }
                    Tag::TableRow => {
                        current_row.clear();
                    }
                    Tag::TableCell => {
                        current_text.clear();
                    }
                    Tag::Image { dest_url, title, .. } => {
                        // Images are inline but we treat them as blocks
                        let alt = current_text.clone();
                        blocks.push(ParsedBlock::Image {
                            alt,
                            url: dest_url.to_string(),
                            title: if title.is_empty() {
                                None
                            } else {
                                Some(title.to_string())
                            },
                            range: range.clone(),
                        });
                    }
                    _ => {}
                }
            }

            Event::End(tag_end) => {
                match tag_end {
                    TagEnd::Heading(level) => {
                        blocks.push(ParsedBlock::Heading {
                            level: level as u8,
                            text: current_text.clone(),
                            range: block_start..range.end,
                        });
                    }
                    TagEnd::Paragraph => {
                        // Check if this paragraph contains only a wiki link placeholder
                        if let Some((target, display)) =
                            find_wiki_link_in_text(&current_text)
                        {
                            blocks.push(ParsedBlock::WikiLink {
                                target,
                                display,
                                range: block_start..range.end,
                            });
                        } else {
                            blocks.push(ParsedBlock::Paragraph {
                                text: current_text.clone(),
                                range: block_start..range.end,
                            });
                        }
                    }
                    TagEnd::CodeBlock => {
                        blocks.push(ParsedBlock::CodeBlock {
                            lang: code_lang.take(),
                            code: current_text.clone(),
                            range: block_start..range.end,
                        });
                    }
                    TagEnd::List(_) => {
                        blocks.push(ParsedBlock::List {
                            items: list_items.clone(),
                            ordered: list_ordered,
                            start: list_start,
                            range: block_start..range.end,
                        });
                    }
                    TagEnd::Item => {
                        list_items.push(ListItem {
                            text: current_item_text.clone(),
                            checkbox: item_checkbox,
                            children: Vec::new(),
                        });
                        in_list_item = false;
                    }
                    TagEnd::BlockQuote(_) => {
                        blocks.push(ParsedBlock::BlockQuote {
                            content: vec![ParsedBlock::Paragraph {
                                text: current_text.clone(),
                                range: block_start..range.end,
                            }],
                            range: block_start..range.end,
                        });
                    }
                    TagEnd::Table => {
                        blocks.push(ParsedBlock::Table {
                            headers: table_headers.clone(),
                            rows: table_rows.clone(),
                            range: block_start..range.end,
                        });
                    }
                    TagEnd::TableHead => {
                        table_headers = current_row.clone();
                        in_table_head = false;
                    }
                    TagEnd::TableRow => {
                        if !in_table_head {
                            table_rows.push(current_row.clone());
                        }
                    }
                    TagEnd::TableCell => {
                        current_row.push(TableCell {
                            content: current_text.clone(),
                            alignment: TableAlignment::Left,
                        });
                    }
                    _ => {}
                }
            }

            Event::Text(text) => {
                if in_list_item {
                    current_item_text.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            }

            Event::Code(code) => {
                if in_list_item {
                    current_item_text.push('`');
                    current_item_text.push_str(&code);
                    current_item_text.push('`');
                } else {
                    current_text.push('`');
                    current_text.push_str(&code);
                    current_text.push('`');
                }
            }

            Event::SoftBreak | Event::HardBreak => {
                if in_list_item {
                    current_item_text.push('\n');
                } else {
                    current_text.push('\n');
                }
            }

            Event::Rule => {
                blocks.push(ParsedBlock::HorizontalRule {
                    range: range.clone(),
                });
            }

            Event::TaskListMarker(checked) => {
                item_checkbox = Some(checked);
            }

            _ => {}
        }

        current_pos = range.end;
    }

    // Add any remaining content as a paragraph
    if current_pos < content.len() {
        let remaining = content[current_pos..].trim();
        if !remaining.is_empty() {
            blocks.push(ParsedBlock::Paragraph {
                text: remaining.to_string(),
                range: current_pos..content.len(),
            });
        }
    }

    blocks
}

/// Extract wiki links from content and return processed content with placeholders
fn extract_wiki_links(content: &str) -> (String, Vec<(String, Option<String>)>) {
    let mut result = content.to_string();
    let mut links = Vec::new();

    // Match [[target]] or [[target|display]]
    let re = regex_lite::Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();

    for cap in re.captures_iter(content) {
        let target = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let display = cap.get(2).map(|m| m.as_str().to_string());

        links.push((target, display));
    }

    (result, links)
}

/// Find a wiki link in text by checking pattern
fn find_wiki_link_in_text(text: &str) -> Option<(String, Option<String>)> {
    // We detect wiki links by pattern matching
    let re = regex_lite::Regex::new(r"^\[\[([^\]|]+)(?:\|([^\]]+))?\]\]$").unwrap();

    if let Some(cap) = re.captures(text.trim()) {
        let target = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let display = cap.get(2).map(|m| m.as_str().to_string());
        return Some((target, display));
    }

    None
}

/// Find the block containing a given byte position
pub fn find_block_at_position(blocks: &[ParsedBlock], byte_pos: usize) -> Option<usize> {
    blocks.iter().position(|block| block.contains(byte_pos))
}

/// Parse inline wiki links from text, returning spans with their types
#[derive(Debug, Clone)]
pub enum InlineSpan {
    Text(String),
    WikiLink { target: String, display: Option<String> },
    Code(String),
    Bold(String),
    Italic(String),
    Link { text: String, url: String },
}

/// Parse inline elements from text
pub fn parse_inline(text: &str) -> Vec<InlineSpan> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for wiki link [[...]]
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            // Flush current text
            if !current_text.is_empty() {
                spans.push(InlineSpan::Text(current_text.clone()));
                current_text.clear();
            }

            // Find closing ]]
            let start = i + 2;
            let mut end = start;
            while end + 1 < chars.len() && !(chars[end] == ']' && chars[end + 1] == ']') {
                end += 1;
            }

            if end + 1 < chars.len() {
                let link_content: String = chars[start..end].iter().collect();
                let parts: Vec<&str> = link_content.splitn(2, '|').collect();
                let target = parts[0].to_string();
                let display = parts.get(1).map(|s| s.to_string());

                spans.push(InlineSpan::WikiLink { target, display });
                i = end + 2;
                continue;
            }
        }

        // Check for inline code `...`
        if chars[i] == '`' {
            // Flush current text
            if !current_text.is_empty() {
                spans.push(InlineSpan::Text(current_text.clone()));
                current_text.clear();
            }

            let start = i + 1;
            let mut end = start;
            while end < chars.len() && chars[end] != '`' {
                end += 1;
            }

            if end < chars.len() {
                let code: String = chars[start..end].iter().collect();
                spans.push(InlineSpan::Code(code));
                i = end + 1;
                continue;
            }
        }

        current_text.push(chars[i]);
        i += 1;
    }

    // Flush remaining text
    if !current_text.is_empty() {
        spans.push(InlineSpan::Text(current_text));
    }

    spans
}
