use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use anyhow::{Result, Context};
use directories::ProjectDirs;

// Defaults
const DEFAULT_MAX_HISTORY: usize = 50;
const DEFAULT_THEME: &str = "dark";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub max_history: usize,
    pub theme: String, // "dark" or "light"
    pub start_login: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_history: DEFAULT_MAX_HISTORY,
            theme: DEFAULT_THEME.to_string(),
            start_login: false,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = get_config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content).unwrap_or_default();
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = get_config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryItem {
    pub content: String, // Text content OR Path to image file
    pub timestamp: DateTime<Local>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default = "default_kind")]
    pub kind: String, // "text" or "image"
    #[serde(default)]
    pub hash: Option<String>, // For image deduplication
}

fn default_kind() -> String {
    "text".to_string()
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ClipboardStorage {
    pub history: Vec<HistoryItem>,
}

impl ClipboardStorage {
    pub fn load() -> Result<Self> {
        let path = get_data_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        // Use from_str directly to propagate deserialization errors
        let storage = serde_json::from_str(&content)?;
        Ok(storage)
    }

    pub fn save(&self) -> Result<()> {
        let path = get_data_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_images_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "asthetic", "clipboard")
            .context("Could not determine project directories")?;
        let dir = proj_dirs.data_dir().join("images");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn add(&mut self, content: String, kind: String, hash: Option<String>, max_history: usize) {
        if kind == "text" {
            // Remove existing identical item
             self.history.retain(|item| item.content != content);
        } else if kind == "image" {
            // Deduplicate by hash if available
             if let Some(ref h) = hash {
                 self.history.retain(|item| item.hash.as_ref() != Some(h));
             }
        }
        
        // Add new item to front
        self.history.insert(0, HistoryItem {
            content,
            timestamp: Local::now(),
            pinned: false,
            kind,
            hash,
        });

        // Limit history size
        if self.history.len() > max_history {
            // Keep pinned items, remove oldest unpinned
            let mut pinned: Vec<HistoryItem> = self.history.iter().filter(|i| i.pinned).cloned().collect();
            let mut unpinned: Vec<HistoryItem> = self.history.iter().filter(|i| !i.pinned).cloned().collect();
            
            // Trim unpinned
            let space_for_unpinned = if max_history > pinned.len() { max_history - pinned.len() } else { 0 };
            unpinned.truncate(space_for_unpinned);
            
            let mut new_history: Vec<HistoryItem> = Vec::new();
            
            // We want to reconstruct the history list preserving the original order (newest first)
            // but only keeping items that survived the truncation (pinned + top unpinned).
            
            // Optimization: unpinned now contains the truncated list of unpinned items we want to keep.
            // Pinned contains all pinned items.
            // We can just iterate the original list again and keep items if they are in our 'keep' sets.
            // But checking existence is slow. A better way is:
            // Since we just want valid items, and the order in `pinned` and `unpinned` is already correct 
            // relative to their own groups (filter preserves order), we just need to merge them back 
            // respecting original relative order.
            
            // Actually, simplest correct way that preserves exact original interleaving:
            let mut unpinned_remaining = space_for_unpinned;
            for item in &self.history {
                if item.pinned {
                    new_history.push(item.clone());
                } else if unpinned_remaining > 0 {
                    new_history.push(item.clone());
                    unpinned_remaining -= 1;
                }
            }
            // self.history = new_history; // This is set below loop
             // Reconstruct preserved history 
             // Order matters: we want to keep the original order.
             // Simplest way: iterate original, keep if pinned OR if it is in the allowed unpinned set.
             // But 'unpinned' vec contains the ones we want to keep.
             
            let unpinned_hashes: std::collections::HashSet<String> = unpinned.iter().map(|i| i.content.clone() /* content is unique-ish for text, but images? hash? */).collect();
            // Wait, this is getting complex.
            // Let's stick to previous logic but parameterized.
            
            let mut new_history = Vec::new();
            let mut count = 0;
            
             for item in &self.history {
                if item.pinned || count < space_for_unpinned {
                    new_history.push(item.clone());
                    if !item.pinned {
                        count += 1;
                    }
                }
            }
            self.history = new_history;
        }
    }
}

fn get_data_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "asthetic", "clipboard")
        .context("Could not determine project directories")?;
    Ok(proj_dirs.data_dir().join("history.json"))
}

fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "asthetic", "clipboard")
        .context("Could not determine project directories")?;
    // Use config_dir for config, but for simplicity let's stick to data_dir or actually follow XDG
    // ProjectDirs::config_dir() is correct for config.
    Ok(proj_dirs.config_dir().join("config.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_limit() {
        let mut storage = ClipboardStorage::default();
        for i in 0..60 {
            storage.add(format!("content {}", i), "text".to_string(), None, 50);
        }
        // Should be capped at 50 (unpinned)
        assert_eq!(storage.history.len(), 50);
        // Newest should be at 0
        assert_eq!(storage.history[0].content, "content 59");
    }

    #[test]
    fn test_pinning_logic() {
        let mut storage = ClipboardStorage::default();
        
        // Add item and pin it
        storage.add("pinned item".to_string(), "text".to_string(), None, 50);
        storage.history[0].pinned = true;
        
        // Add 60 more items
        for i in 0..60 {
            storage.add(format!("content {}", i), "text".to_string(), None, 50);
        }

        // Pinned item should still be there
        assert!(storage.history.iter().any(|i| i.content == "pinned item"));
        
        // Size logic: 1 pinned + 50 unpinned allowed = 51.
        assert_eq!(storage.history.len(), 51);
        
        let count_pinned = storage.history.iter().filter(|i| i.pinned).count();
        let count_unpinned = storage.history.iter().filter(|i| !i.pinned).count();
        assert_eq!(count_pinned, 1);
        assert_eq!(count_unpinned, 50);
    }
}
