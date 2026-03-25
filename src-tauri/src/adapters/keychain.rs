use log::{info, warn};

const SERVICE_NAME: &str = "com.memorypalace.app";

/// OS keychain adapter for secure secret storage.
/// Uses Windows Credential Manager, macOS Keychain, or Linux Secret Service.
/// Falls back gracefully if the keychain is unavailable.
pub struct KeychainAdapter {
    available: bool,
}

impl KeychainAdapter {
    pub fn new() -> Self {
        // Test availability by doing a no-op probe
        let available = keyring::Entry::new(SERVICE_NAME, "__probe__")
            .is_ok();
        if available {
            info!("OS keychain available for secret storage");
        } else {
            warn!("OS keychain unavailable — secrets will use config store fallback");
        }
        Self { available }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Store a secret in the OS keychain.
    pub fn store_secret(&self, key: &str, value: &str) -> Result<(), String> {
        if !self.available {
            return Err("Keychain not available".into());
        }
        let entry = keyring::Entry::new(SERVICE_NAME, key)
            .map_err(|e| format!("Keychain entry error: {}", e))?;
        entry
            .set_password(value)
            .map_err(|e| format!("Failed to store secret: {}", e))?;
        Ok(())
    }

    /// Retrieve a secret from the OS keychain.
    pub fn get_secret(&self, key: &str) -> Result<Option<String>, String> {
        if !self.available {
            return Ok(None);
        }
        let entry = keyring::Entry::new(SERVICE_NAME, key)
            .map_err(|e| format!("Keychain entry error: {}", e))?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("Failed to retrieve secret: {}", e)),
        }
    }

    /// Delete a secret from the OS keychain.
    pub fn delete_secret(&self, key: &str) -> Result<(), String> {
        if !self.available {
            return Ok(());
        }
        let entry = keyring::Entry::new(SERVICE_NAME, key)
            .map_err(|e| format!("Keychain entry error: {}", e))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // already gone
            Err(e) => Err(format!("Failed to delete secret: {}", e)),
        }
    }
}

/// Keychain key constants for API secrets.
pub mod keys {
    pub const CLAUDE_API_KEY: &str = "claude_api_key";
    pub const OPENAI_COMPAT_API_KEY: &str = "openai_compat_api_key";
}
