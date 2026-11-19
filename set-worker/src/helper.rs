use worker::{Error, Result};

/// Generates a 128-bit random token as a 32-character hex string.
pub(crate) fn new_token() -> Result<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| Error::RustError(e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
