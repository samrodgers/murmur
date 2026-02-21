//! Proof-of-work challenge for registration.
//! Phase 1: Stubbed out. PoW is not required for MVP.
//! In a full implementation, registration requires finding a nonce such that
//! SHA-256(pubkey || nonce) has N leading zero bits.

use sha2::{Digest, Sha256};

/// Difficulty: number of leading zero bits required.
pub const DEFAULT_DIFFICULTY: u32 = 20; // ~1M hashes, ~seconds on modern hardware

/// Compute a proof-of-work for a given pubkey.
/// Returns the nonce that satisfies the difficulty requirement.
pub fn compute_pow(pubkey: &[u8; 32], difficulty: u32) -> u64 {
    let mut nonce: u64 = 0;
    loop {
        let mut hasher = Sha256::new();
        hasher.update(pubkey);
        hasher.update(nonce.to_le_bytes());
        let hash = hasher.finalize();
        if has_leading_zeros(&hash, difficulty) {
            return nonce;
        }
        nonce += 1;
    }
}

/// Verify a proof-of-work.
pub fn verify_pow(pubkey: &[u8; 32], nonce: u64, difficulty: u32) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    hasher.update(nonce.to_le_bytes());
    let hash = hasher.finalize();
    has_leading_zeros(&hash, difficulty)
}

fn has_leading_zeros(hash: &[u8], bits: u32) -> bool {
    let full_bytes = (bits / 8) as usize;
    let remaining_bits = bits % 8;

    for byte in &hash[..full_bytes] {
        if *byte != 0 {
            return false;
        }
    }

    if remaining_bits > 0 && full_bytes < hash.len() {
        let mask = 0xFF << (8 - remaining_bits);
        if hash[full_bytes] & mask != 0 {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_low_difficulty() {
        let pubkey = [0u8; 32];
        let nonce = compute_pow(&pubkey, 8); // Very easy
        assert!(verify_pow(&pubkey, nonce, 8));
    }
}
