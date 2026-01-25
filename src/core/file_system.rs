//! File system operations and file tree management

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use walkdir::WalkDir;

/// Represents a file or directory in the tree
#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
    pub modified: Option<SystemTime>,
}

impl FileNode {
    /// Create a new file node
    pub fn new(path: PathBuf, is_dir: bool) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let modified = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());

        Self {
            name,
            path,
            is_dir,
            children: Vec::new(),
            expanded: false,
            modified,
        }
    }

    /// Check if this is a markdown file
    pub fn is_markdown(&self) -> bool {
        !self.is_dir
            && self
                .path
                .extension()
                .map(|ext| ext == "md" || ext == "markdown")
                .unwrap_or(false)
    }

    /// Sort children: directories first, then files, alphabetically
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

/// File tree representing a vault structure
#[derive(Debug, Clone, Default)]
pub struct FileTree {
    pub root: Option<FileNode>,
    pub root_path: Option<PathBuf>,
}

impl FileTree {
    /// Create a file tree from a directory path
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut root = FileNode::new(path.to_path_buf(), true);
        root.expanded = true;

        Self::build_tree(&mut root, path, 0, 10)?;
        root.sort_children();

        Ok(Self {
            root: Some(root),
            root_path: Some(path.to_path_buf()),
        })
    }

    /// Recursively build the file tree
    fn build_tree(node: &mut FileNode, path: &Path, depth: usize, max_depth: usize) -> Result<()> {
        if depth >= max_depth {
            return Ok(());
        }

        let entries = std::fs::read_dir(path)?;

        for entry in entries.flatten() {
            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Skip hidden files and directories
            if file_name.starts_with('.') {
                continue;
            }

            // Skip common non-content directories
            if file_name == "node_modules" || file_name == "target" || file_name == ".git" {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let mut child = FileNode::new(entry_path.clone(), is_dir);

            if is_dir {
                Self::build_tree(&mut child, &entry_path, depth + 1, max_depth)?;
            }

            node.children.push(child);
        }

        Ok(())
    }

    /// Refresh the file tree
    pub fn refresh(&mut self) -> Result<()> {
        if let Some(ref root_path) = self.root_path.clone() {
            *self = Self::from_path(root_path)?;
        }
        Ok(())
    }

    /// Find a node by path
    pub fn find_node(&self, path: &Path) -> Option<&FileNode> {
        self.root.as_ref().and_then(|root| Self::find_in_node(root, path))
    }

    fn find_in_node<'a>(node: &'a FileNode, path: &Path) -> Option<&'a FileNode> {
        if node.path == path {
            return Some(node);
        }

        for child in &node.children {
            if let Some(found) = Self::find_in_node(child, path) {
                return Some(found);
            }
        }

        None
    }

    /// Toggle expansion state of a directory
    pub fn toggle_expanded(&mut self, path: &Path) {
        if let Some(ref mut root) = self.root {
            Self::toggle_in_node(root, path);
        }
    }

    fn toggle_in_node(node: &mut FileNode, path: &Path) {
        if node.path == path {
            node.expanded = !node.expanded;
            return;
        }

        for child in &mut node.children {
            Self::toggle_in_node(child, path);
        }
    }
}

/// Create a new file in the vault
pub fn create_file(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, "")?;
    Ok(())
}

/// Create a new directory in the vault
pub fn create_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Delete a file or directory
pub fn delete(path: &Path) -> Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Rename a file or directory
pub fn rename(from: &Path, to: &Path) -> Result<()> {
    std::fs::rename(from, to)?;
    Ok(())
}

/// Get all markdown files in a directory recursively
pub fn get_markdown_files(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "md" || ext == "markdown")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}
