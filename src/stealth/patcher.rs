use std::path::Path;
use std::fs;
use regex::bytes::Regex;
use crate::error::{XcelerateResult, XcelerateError};

pub struct BinaryPatcher;

impl BinaryPatcher {
    /// Patches the binary at the given path to remove/replace automation detection strings.
    pub fn patch_binary(path: &Path) -> XcelerateResult<()> {
        if !path.exists() {
            return Err(XcelerateError::NotFound(format!("Binary not found for patching: {:?}", path)));
        }

        let mut content = fs::read(path).map_err(|e| XcelerateError::NotFound(format!("Failed to read binary: {}", e)))?;
        
        // 1. Search for the CDC pattern: {window.cdc...;}
        // This is a common pattern used by anti-bot scripts to detect chromedriver.
        // We use a regex to find the block and replace it with a neutral payload of the same length.
        let re = Regex::new(r"\{window\.cdc.*?;\}").map_err(|_| XcelerateError::InternalError)?;
        
        let mut patched = false;
        
        // We iterate and replace. Using find_iter to handle multiple occurrences if any.
        // However, for the first pass, we can do a simple range replacement.
        // Note: We MUST NOT change the file size, so we ljust with spaces or similar.
        
        if let Some(m) = re.find(&content) {
            let start = m.start();
            let end = m.end();
            let len = end - start;
            
            // Replacement payload (equivalent to console.log but same length)
            let mut payload = b"{console.log(\"xcelerate stealth active!\")}".to_vec();
            if payload.len() > len {
                // Truncate if somehow the payload is longer (unlikely for typical CDC blocks)
                payload.truncate(len);
            } else {
                // Pad with spaces to match length exactly
                payload.extend(std::iter::repeat(b' ').take(len - payload.len()));
            }
            
            content[start..end].copy_from_slice(&payload);
            patched = true;
        }

        if patched {
            fs::write(path, content).map_err(|_e| XcelerateError::InternalError)?;
            eprintln!("[PATCHER] Successfully patched binary: {:?}", path);
        } else {
            eprintln!("[PATCHER] No detection patterns found or already patched: {:?}", path);
        }

        Ok(())
    }
}
