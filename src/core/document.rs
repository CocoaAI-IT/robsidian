//! Document management for markdown files

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A markdown document
#[derive(Debug, Clone)]
pub struct Document {
    /// File path
    pub path: PathBuf,
    /// Document content
    pub content: String,
    /// Whether the document has unsaved changes
    pub modified: bool,
    /// Last modification time
    pub last_modified: Option<SystemTime>,
    /// Document metadata (YAML frontmatter)
    pub metadata: DocumentMetadata,
}

/// Document metadata from YAML frontmatter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl Document {
    /// Create a new empty document
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            content: String::new(),
            modified: false,
            last_modified: None,
            metadata: DocumentMetadata::default(),
        }
    }

    /// Open a document from a file
    pub fn open(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let last_modified = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok());

        let metadata = Self::parse_frontmatter(&content).unwrap_or_default();

        Ok(Self {
            path: path.to_path_buf(),
            content,
            modified: false,
            last_modified,
            metadata,
        })
    }

    /// Save the document to disk
    pub fn save(&self) -> Result<()> {
        fs::write(&self.path, &self.content)
            .with_context(|| format!("Failed to save file: {}", self.path.display()))?;
        tracing::info!("Saved document: {}", self.path.display());
        Ok(())
    }

    /// Save the document and update modified flag
    pub fn save_mut(&mut self) -> Result<()> {
        self.save()?;
        self.modified = false;
        self.last_modified = Some(SystemTime::now());
        Ok(())
    }

    /// Get the document title (filename without extension or metadata title)
    pub fn title(&self) -> String {
        self.metadata.title.clone().unwrap_or_else(|| {
            self.path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        })
    }

    /// Parse YAML frontmatter from content
    fn parse_frontmatter(content: &str) -> Option<DocumentMetadata> {
        if !content.starts_with("---") {
            return None;
        }

        let end = content[3..].find("---")?;
        let frontmatter = &content[3..3 + end].trim();

        serde_json::from_str(frontmatter).ok().or_else(|| {
            // Try simple key-value parsing
            let mut metadata = DocumentMetadata::default();
            for line in frontmatter.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');
                    match key {
                        "title" => metadata.title = Some(value.to_string()),
                        "tags" => {
                            metadata.tags = value
                                .trim_matches(|c| c == '[' || c == ']')
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .collect();
                        }
                        _ => {}
                    }
                }
            }
            Some(metadata)
        })
    }

    /// Get content without frontmatter for preview
    pub fn content_without_frontmatter(&self) -> &str {
        if !self.content.starts_with("---") {
            return &self.content;
        }

        if let Some(end) = self.content[3..].find("---") {
            let after_frontmatter = 3 + end + 3;
            if after_frontmatter < self.content.len() {
                return self.content[after_frontmatter..].trim_start();
            }
        }

        &self.content
    }

    /// Update content and mark as modified
    pub fn set_content(&mut self, content: String) {
        if self.content != content {
            self.content = content;
            self.modified = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: "Test Document"
tags: [rust, markdown]
---

# Content here
"#;
        let metadata = Document::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.title, Some("Test Document".to_string()));
    }
}
