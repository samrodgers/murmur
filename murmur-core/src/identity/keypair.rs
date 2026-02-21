use ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey, SECRET_KEY_LENGTH,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Failed to read key file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid key data")]
    InvalidKey,
    #[error("Decryption failed — wrong passphrase?")]
    DecryptionFailed,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A user's cryptographic identity — an Ed25519 keypair.
/// The secret key never leaves this struct except for encrypted export.
pub struct Identity {
    signing_key: SigningKey,
}

/// Serializable public key wrapper for use in events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PubKey(#[serde(with = "hex_bytes")] pub [u8; 32]);

impl Identity {
    /// Generate a brand new identity with a random keypair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Sign arbitrary bytes and return the signature.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Get the public key as bytes.
    pub fn pubkey(&self) -> PubKey {
        PubKey(self.signing_key.verifying_key().to_bytes())
    }

    /// Get the public key as a hex string for display.
    pub fn pubkey_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }

    /// Get a short display form of the pubkey (first 16 hex chars).
    pub fn pubkey_short(&self) -> String {
        let full = self.pubkey_hex();
        format!("{}…", &full[..16])
    }

    /// Save the identity to a file. In Phase 1, we use a simple
    /// passphrase-derived XOR obfuscation. A real implementation
    /// would use proper key derivation (argon2) + AES-GCM.
    pub fn save(&self, path: &Path, passphrase: &str) -> Result<(), IdentityError> {
        let secret_bytes = self.signing_key.to_bytes();
        let key_hash = derive_key(passphrase);
        let encrypted: Vec<u8> = secret_bytes
            .iter()
            .zip(key_hash.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();

        let stored = StoredIdentity {
            encrypted_secret: hex::encode(&encrypted),
            pubkey: self.pubkey_hex(),
        };
        let json = serde_json::to_string_pretty(&stored)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load an identity from an encrypted file.
    pub fn load(path: &Path, passphrase: &str) -> Result<Self, IdentityError> {
        let json = std::fs::read_to_string(path)?;
        let stored: StoredIdentity = serde_json::from_str(&json)?;
        let encrypted =
            hex::decode(&stored.encrypted_secret).map_err(|_| IdentityError::InvalidKey)?;

        let key_hash = derive_key(passphrase);
        let decrypted: Vec<u8> = encrypted
            .iter()
            .zip(key_hash.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();

        let secret_bytes: [u8; SECRET_KEY_LENGTH] = decrypted
            .try_into()
            .map_err(|_| IdentityError::InvalidKey)?;

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let identity = Self { signing_key };

        // Verify the pubkey matches what was stored
        if identity.pubkey_hex() != stored.pubkey {
            return Err(IdentityError::DecryptionFailed);
        }

        Ok(identity)
    }
}

impl PubKey {
    /// Create from a hex string.
    pub fn from_hex(s: &str) -> Result<Self, IdentityError> {
        let bytes = hex::decode(s).map_err(|_| IdentityError::InvalidKey)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| IdentityError::InvalidKey)?;
        Ok(PubKey(arr))
    }

    /// Display as hex.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Short display form.
    pub fn short(&self) -> String {
        let full = self.to_hex();
        format!("{}…", &full[..16])
    }
}

/// Verify a signature against a public key.
pub fn verify(pubkey: &PubKey, message: &[u8], signature: &[u8; 64]) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pubkey.0) else {
        return false;
    };
    let sig = Signature::from_bytes(signature);
    verifying_key.verify(message, &sig).is_ok()
}

/// Simple passphrase -> key derivation (SHA-256). Not production-grade;
/// a real implementation would use argon2id.
fn derive_key(passphrase: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.finalize().to_vec()
}

#[derive(Serialize, Deserialize)]
struct StoredIdentity {
    encrypted_secret: String,
    pubkey: String,
}

/// Serde helper for [u8; 32] as hex strings.
mod hex_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 32 bytes"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_sign() {
        let id = Identity::generate();
        let message = b"hello world";
        let sig = id.sign(message);
        let sig_bytes: [u8; 64] = sig.to_bytes();
        assert!(verify(&id.pubkey(), message, &sig_bytes));
    }

    #[test]
    fn test_wrong_message_fails_verify() {
        let id = Identity::generate();
        let sig = id.sign(b"hello");
        let sig_bytes: [u8; 64] = sig.to_bytes();
        assert!(!verify(&id.pubkey(), b"world", &sig_bytes));
    }

    #[test]
    fn test_pubkey_hex_roundtrip() {
        let id = Identity::generate();
        let hex = id.pubkey_hex();
        let pk = PubKey::from_hex(&hex).unwrap();
        assert_eq!(pk, id.pubkey());
    }
}
