use std::path::Path;
use std::fs;
use regex::bytes::Regex;
use crate::error::{XcelerateResult, XcelerateError};

pub struct BinaryPatcher;

impl BinaryPatcher {
    /// Patches a copy of the binary at the given path and returns the path to the patched version.
    /// This prevents modifying the original user executable while maintaining dependency integrity.
    pub fn patch_to_temp(path: &Path) -> XcelerateResult<std::path::PathBuf> {
        if !path.exists() {
            return Err(XcelerateError::NotFound(format!("Binary not found for patching: {:?}", path)));
        }

        // Try to create a copy in the SAME directory to preserve side-by-side dependencies
        let file_name = path.file_name().ok_or(XcelerateError::InternalError)?;
        let mut patch_path = path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir());
        
        patch_path.push(format!("xcelerate_{}", file_name.to_string_lossy()));
        
        match fs::copy(path, &patch_path) {
            Ok(_) => {
                let mut content = fs::read(&patch_path).map_err(|e| XcelerateError::NotFound(format!("Failed to read binary: {}", e)))?;
                let re = Regex::new(r"\{window\.cdc.*?;\}").map_err(|_| XcelerateError::InternalError)?;
                
                if let Some(m) = re.find(&content) {
                    let start = m.start();
                    let end = m.end();
                    let len = end - start;
                    
                    let mut payload = b"{console.log(\"xcelerate stealth active!\")}".to_vec();
                    if payload.len() > len {
                        payload.truncate(len);
                    } else {
                        payload.extend(std::iter::repeat(b' ').take(len - payload.len()));
                    }
                    
                    content[start..end].copy_from_slice(&payload);
                    fs::write(&patch_path, content).map_err(|_e| XcelerateError::InternalError)?;
                }
                Ok(patch_path)
            }
            Err(_e) => {
                Ok(path.to_path_buf())
            }
        }
    }
}
