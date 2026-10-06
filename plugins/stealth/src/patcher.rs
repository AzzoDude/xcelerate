use std::fs;
use std::path::Path;

use crate::{Error, Result};

/// Chrome's automation banner starts with this literal. It used to be matched
/// with the `regex` crate as `\{window\.cdc.*?;\}`, but that is a plain byte
/// scan (`.` never crosses a newline, `.*?` is the shortest run up to the
/// terminator), so it is done by hand here. That drops a multi-hundred-kilobyte
/// dependency from the shipped library for a one-shot patch of a cold binary -
/// and is faster than the regex prefilter for a single literal pattern.
const CDC_MARKER: &[u8] = b"{window.cdc";
const CDC_TERMINATOR: &[u8] = b";}";

pub struct BinaryPatcher;

impl BinaryPatcher {
    /// Patches a copy of the binary at the given path and returns the path to the patched version.
    /// This prevents modifying the original user executable while maintaining dependency integrity.
    pub fn patch_to_temp(path: &Path) -> Result<std::path::PathBuf> {
        if !path.exists() {
            return Err(Error::NotFound(format!(
                "Binary not found for patching: {:?}",
                path
            )));
        }

        // Try to create a copy in the SAME directory to preserve side-by-side dependencies
        let file_name = path.file_name().ok_or(Error::Internal)?;
        let mut patch_path = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        patch_path.push(format!("xcelerate_{}", file_name.to_string_lossy()));

        match fs::copy(path, &patch_path) {
            Ok(_) => {
                let mut content = fs::read(&patch_path)
                    .map_err(|e| Error::NotFound(format!("Failed to read binary: {}", e)))?;

                if let Some((start, end)) = find_cdc_span(&content) {
                    let len = end - start;

                    // Overwrite in place, padding or truncating so the binary's
                    // byte length (and therefore its layout) is unchanged.
                    let mut payload = b"{console.log(\"xcelerate stealth active!\")}".to_vec();
                    if payload.len() > len {
                        payload.truncate(len);
                    } else {
                        payload.extend(std::iter::repeat_n(b' ', len - payload.len()));
                    }

                    content[start..end].copy_from_slice(&payload);
                    fs::write(&patch_path, content).map_err(|_e| Error::Internal)?;
                }
                Ok(patch_path)
            }
            Err(_e) => Ok(path.to_path_buf()),
        }
    }
}

/// Byte range of the first `{window.cdc...;}` run, mirroring the removed regex's
/// `.`-does-not-match-`\n` and non-greedy `.*?` semantics.
fn find_cdc_span(content: &[u8]) -> Option<(usize, usize)> {
    let mut from = 0;
    while let Some(offset) = find_subslice(&content[from..], CDC_MARKER) {
        let start = from + offset;
        let rest = &content[start..];
        // A `.` in the old pattern excluded `\n`, so the terminator has to be on
        // the same line as the marker.
        let line_end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
        if let Some(term) = find_subslice(&rest[..line_end], CDC_TERMINATOR) {
            return Some((start, start + term + CDC_TERMINATOR.len()));
        }
        from = start + 1;
    }
    None
}

/// Offset of the first `needle` in `haystack`. `needle` must be non-empty.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    debug_assert!(!needle.is_empty());
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::find_cdc_span;

    #[test]
    fn finds_the_automation_banner() {
        // The terminator is the literal `;}` (not `)}`), matching the old regex.
        let data = b"pre{window.cdc_adoQpoasnfa = 1; console.log(1);}post";
        let (start, end) = find_cdc_span(data).expect("span");
        assert_eq!(
            &data[start..end],
            b"{window.cdc_adoQpoasnfa = 1; console.log(1);}"
        );
    }

    #[test]
    fn stops_at_the_first_terminator() {
        let data = b"{window.cdc_x;}tail;}";
        let (start, end) = find_cdc_span(data).expect("span");
        assert_eq!(&data[start..end], b"{window.cdc_x;}");
    }

    #[test]
    fn does_not_cross_a_newline() {
        // The old `.` excluded `\n`, so a terminator on a later line must not match.
        assert_eq!(find_cdc_span(b"{window.cdc_x\n;}"), None);
    }

    #[test]
    fn returns_none_without_a_marker() {
        assert_eq!(find_cdc_span(b"window.cdc_x;}"), None);
        assert_eq!(find_cdc_span(b"nothing here"), None);
    }
}
