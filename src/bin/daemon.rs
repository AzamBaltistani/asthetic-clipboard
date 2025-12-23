use asthetic_clipboard::{ClipboardStorage, AppConfig};
use arboard::Clipboard;
use std::{thread, time::Duration};
use anyhow::Result;
use sha2::{Digest, Sha256};
// use std::borrow::Cow;
// use hex; // Implicit via format!

// Helper to load with retry (with exponential backoff)
fn load_storage_with_retry() -> ClipboardStorage {
    let mut attempts = 0;
    let max_attempts = 5;
    
    loop {
        match ClipboardStorage::load() {
            Ok(s) => return s,
            Err(e) => {
                attempts += 1;
                
                // Only log if it's not a lock contention issue
                let err_msg = e.to_string();
                if !err_msg.contains("Failed to acquire") {
                    eprintln!("Error loading storage (attempt {}): {}", attempts, e);
                }
                
                if attempts >= max_attempts {
                    eprintln!("Failed to load storage after {} attempts. Returning default.", max_attempts);
                    return ClipboardStorage::default();
                }
                
                // Exponential backoff: 50ms, 100ms, 200ms, 400ms, 800ms
                let delay_ms = 50 * (1 << (attempts - 1));
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }
}

// Helper to save with retry (with exponential backoff)
fn save_storage_with_retry(storage: &ClipboardStorage) -> Result<()> {
    let mut attempts = 0;
    let max_attempts = 5;
    
    loop {
        match storage.save() {
            Ok(()) => return Ok(()),
            Err(e) => {
                attempts += 1;
                
                // Only log if it's not a lock contention issue
                let err_msg = e.to_string();
                if !err_msg.contains("Failed to acquire") {
                    eprintln!("Error saving storage (attempt {}): {}", attempts, e);
                }
                
                if attempts >= max_attempts {
                    return Err(e);
                }
                
                // Exponential backoff: 50ms, 100ms, 200ms, 400ms, 800ms
                let delay_ms = 50 * (1 << (attempts - 1));
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }
}

fn main() -> Result<()> {
    let mut clipboard = Clipboard::new()?;
    let mut last_text_content = String::new();
    let mut last_image_hash = String::new();

    // Initial check (optional, let's keep it simple and just start loop)
    println!("Clipboard daemon started...");

    loop {
        thread::sleep(Duration::from_millis(500));

        // 1. Check Text
        match clipboard.get_text() {
            Ok(content) => {
                if content != last_text_content && !content.trim().is_empty() {
                    println!("Detected text change");
                    let mut storage = load_storage_with_retry();
                    let config = AppConfig::load().unwrap_or_default();
                    storage.add(content.clone(), "text".to_string(), None, config.max_history);
                    if let Err(e) = save_storage_with_retry(&storage) {
                        eprintln!("Failed to save history after retries: {}", e);
                    }
                    last_text_content = content;
                    last_image_hash.clear(); 
                    continue; // Skip image check if text was found (optimization)
                }
            },
            Err(_) => {}
        }

        // 2. Check Image
        match clipboard.get_image() {
             Ok(image_data) => {
                 // Compute hash
                 let mut hasher = Sha256::new();
                 hasher.update(&image_data.bytes);
                 let hash = hex::encode(hasher.finalize());

                 if hash != last_image_hash && !hash.is_empty() {
                     println!("Detected image change: {}", hash);
                     
                     // Save Image to Disk
                     if let Ok(images_dir) = ClipboardStorage::get_images_dir() {
                         let file_name = format!("{}.png", hash);
                         let file_path = images_dir.join(&file_name);
                         
                         // Save using image crate
                         if let Err(e) = image::save_buffer(
                             &file_path, 
                             &image_data.bytes, 
                             image_data.width as u32, 
                             image_data.height as u32, 
                             image::ColorType::Rgba8
                         ) {
                             eprintln!("Failed to save image to disk: {}", e);
                         } else {
                             // Add to storage
                             let mut storage = load_storage_with_retry();
                             let config = AppConfig::load().unwrap_or_default();
                             // For images, 'content' is the absolute path to the file
                             storage.add(
                                 file_path.to_string_lossy().to_string(), 
                                 "image".to_string(), 
                                 Some(hash.clone()),
                                 config.max_history
                             );
                             if let Err(e) = save_storage_with_retry(&storage) {
                                 eprintln!("Failed to save history after retries: {}", e);
                             }
                         }
                     }
                     last_image_hash = hash;
                     last_text_content.clear(); // Clear text as clipboard now has image
                 }
             },
             Err(_) => {}
        }
    }
}
