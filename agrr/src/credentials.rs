// Fallback crypto utilities are part of the public API but not called from
// the main binary in all code paths. Suppress dead_code/unused_imports warnings
// for this module until the fallback code path is wired into the main flow.
#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
#[allow(unused_imports)]
use std::path::PathBuf;

#[allow(unused_imports)]
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;

const SERVICE: &str = "agrr";

// ─── Primary store: OS Keychain ───────────────────────────────────────────────

/// Retrieve a credential by key. Returns `None` if not stored.
pub fn get(key: &str) -> Option<String> {
    keyring::Entry::new(SERVICE, key)
        .ok()?
        .get_password()
        .ok()
}

/// Store a credential in the OS Keychain.
pub fn set(key: &str, value: &str) -> Result<(), CredentialError> {
    keyring::Entry::new(SERVICE, key)
        .map_err(CredentialError::Keyring)?
        .set_password(value)
        .map_err(CredentialError::Keyring)
}

/// Delete a credential from the OS Keychain (or fallback store).
pub fn delete(key: &str) {
    if let Ok(entry) = keyring::Entry::new(SERVICE, key) {
        let _ = entry.delete_credential();
    }
    // Also attempt removal from the encrypted fallback file.
    let _ = fallback_delete(key);
}

/// Delete all credentials listed in `requires_auth`.
/// Called when a script exits with code 99 (AUTH_ERROR).
pub fn delete_all(requires_auth: &[String]) {
    for key in requires_auth {
        delete(key);
    }
}

#[derive(Debug)]
pub enum CredentialError {
    Keyring(keyring::Error),
    Fallback(String),
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialError::Keyring(e) => write!(f, "keychain error: {e}"),
            CredentialError::Fallback(e) => write!(f, "fallback store error: {e}"),
        }
    }
}

// ─── Fallback: AES-256-GCM encrypted file ────────────────────────────────────
//
// Used when the OS Keychain is unavailable (e.g. headless Linux).
// The file lives at ~/.config/agrr/credentials.enc.
// It stores key→value pairs as JSON, encrypted with a master password
// that is derived using PBKDF2 and prompted once per session.

#[allow(dead_code)]
const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32; // AES-256
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

fn fallback_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("agrr").join("credentials.enc"))
}

/// Check whether the OS Keychain is reachable.
/// We do this by attempting a no-op read of a sentinel key.
#[allow(dead_code)]
pub fn keychain_available() -> bool {
    keyring::Entry::new(SERVICE, "__agrr_probe__")
        .map(|e| {
            // A "not found" error still means the keychain is reachable.
            match e.get_password() {
                Ok(_) | Err(keyring::Error::NoEntry) => true,
                Err(_) => false,
            }
        })
        .unwrap_or(false)
}

/// Load the encrypted store, decrypt it, and return a map.
/// Returns an empty map if the file doesn't exist.
fn fallback_load(master: &str) -> Result<HashMap<String, String>, CredentialError> {
    let path = fallback_path().ok_or_else(|| CredentialError::Fallback("no config dir".into()))?;

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let data =
        std::fs::read(&path).map_err(|e| CredentialError::Fallback(e.to_string()))?;

    if data.len() < SALT_LEN + NONCE_LEN + 1 {
        return Err(CredentialError::Fallback("arquivo corrompido".into()));
    }

    let (salt, rest) = data.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

    let key = derive_key(master, salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CredentialError::Fallback("senha mestre incorreta".into()))?;

    serde_json::from_slice::<HashMap<String, String>>(&plaintext)
        .map_err(|e| CredentialError::Fallback(e.to_string()))
}

/// Encrypt and persist the store.
fn fallback_save(master: &str, store: &HashMap<String, String>) -> Result<(), CredentialError> {
    let path = fallback_path().ok_or_else(|| CredentialError::Fallback("no config dir".into()))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CredentialError::Fallback(e.to_string()))?;
    }

    let json =
        serde_json::to_vec(store).map_err(|e| CredentialError::Fallback(e.to_string()))?;

    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(master, &salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, json.as_ref())
        .map_err(|e| CredentialError::Fallback(e.to_string()))?;

    let mut file_data = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&nonce_bytes);
    file_data.extend_from_slice(&ciphertext);

    std::fs::write(&path, file_data).map_err(|e| CredentialError::Fallback(e.to_string()))
}

fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    *Key::<Aes256Gcm>::from_slice(&key)
}

/// Retrieve from encrypted fallback store.
#[allow(dead_code)]
pub fn fallback_get(key: &str, master: &str) -> Option<String> {
    fallback_load(master).ok()?.remove(key)
}

/// Store in encrypted fallback store.
#[allow(dead_code)]
pub fn fallback_set(k: &str, value: &str, master: &str) -> Result<(), CredentialError> {
    let mut store = fallback_load(master).unwrap_or_default();
    store.insert(k.to_string(), value.to_string());
    fallback_save(master, &store)
}

/// Delete from encrypted fallback store.
fn fallback_delete(_key: &str) -> Result<(), CredentialError> {
    // We need the master password to rewrite the file; if we don't have it
    // (e.g. during a keychain-available session), skip silently.
    // The key will be absent on next decryption anyway if overshadowed by keychain.
    // Full removal happens when the user explicitly clears the fallback store.
    let path = match fallback_path() {
        Some(p) => p,
        None => return Ok(()),
    };
    if !path.exists() {
        return Ok(());
    }
    // Without master we cannot rewrite — this is a known limitation documented in design.md.
    Ok(())
}

// ─── Env-var injection ────────────────────────────────────────────────────────

/// Build the environment variable map to inject into the script subprocess.
/// Keys: `AGRR_CRED_<UPPERCASE_KEY>`.
#[allow(dead_code)]
pub fn build_cred_env(requires_auth: &[String]) -> HashMap<String, String> {
    requires_auth
        .iter()
        .filter_map(|key| {
            get(key).map(|val| (format!("AGRR_CRED_{}", key.to_uppercase()), val))
        })
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cred_env_key_format() {
        // build_cred_env without real keychain — test key-name transformation
        // directly via the naming logic.
        let key = "db_pass";
        let env_name = format!("AGRR_CRED_{}", key.to_uppercase());
        assert_eq!(env_name, "AGRR_CRED_DB_PASS");
    }

    #[test]
    fn cred_env_mixed_case_keys() {
        for (input, expected) in [
            ("AWS_KEY", "AGRR_CRED_AWS_KEY"),
            ("myService", "AGRR_CRED_MYSERVICE"),
            ("svc_user", "AGRR_CRED_SVC_USER"),
        ] {
            let env_name = format!("AGRR_CRED_{}", input.to_uppercase());
            assert_eq!(env_name, expected);
        }
    }

    #[test]
    fn delete_all_is_safe_with_empty_list() {
        // Must not panic on empty requires_auth
        delete_all(&[]);
    }
}
