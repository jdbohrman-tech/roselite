use crate::error::*;
use std::sync::Arc;

/// Cryptographic operations for package signing and verification
/// Currently uses basic crypto libraries with Veilid-compatible algorithms (Ed25519, BLAKE3)
/// POSSIBLE TODO: Replace with direct Veilid crypto API once available
pub struct CryptoManager {
    initialized: bool,
}

impl CryptoManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            initialized: true,
        })
    }
    
    /// Initialize with Veilid (placeholder for future integration)
    pub async fn init_with_veilid(&mut self, _veilid_api: Arc<veilid_core::VeilidAPI>) -> Result<()> {
        // POSSIBLE TODO: Initialize with actual Veilid crypto system
        self.initialized = true;
        Ok(())
    }
    
    /// Generate a new Ed25519 keypair (VLD0 compatible)
    pub fn generate_keypair(&self) -> Result<(String, String)> {
        if !self.initialized {
            return Err(CryptoError::InitializationFailed("Crypto not initialized".to_string()).into());
        }
        
        // Generate Ed25519 keypair using ed25519-dalek
        use ed25519_dalek::{SigningKey, VerifyingKey};
        use rand::rngs::OsRng;
        
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        
        // Return as hex-encoded strings
        let public_key = hex::encode(verifying_key.to_bytes());
        let secret_key = hex::encode(signing_key.to_bytes());
        
        Ok((public_key, secret_key))
    }
    
    /// Generate a new X25519 keypair for key exchange
    pub fn generate_x25519_keypair(&self) -> Result<(String, String)> {
        if !self.initialized {
            return Err(CryptoError::InitializationFailed("Crypto not initialized".to_string()).into());
        }
        
        use x25519_dalek::{StaticSecret, PublicKey};
        use rand::rngs::OsRng;
        
        let secret_key = StaticSecret::random_from_rng(&mut OsRng);
        let public_key = PublicKey::from(&secret_key);
        
        // Return as hex-encoded strings
        let public_key_hex = hex::encode(public_key.as_bytes());
        let secret_key_hex = hex::encode(secret_key.as_bytes());
        
        Ok((public_key_hex, secret_key_hex))
    }
    
    /// Sign data with Ed25519 private key
    pub fn sign(&self, data: &[u8], private_key: &str) -> Result<String> {
        if !self.initialized {
            return Err(CryptoError::InitializationFailed("Crypto not initialized".to_string()).into());
        }
        
        use ed25519_dalek::{SigningKey, Signature, Signer};
        
        // Parse the secret key from hex
        let secret_bytes = hex::decode(private_key)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid secret key hex: {}", e)))?;
        
        if secret_bytes.len() != 32 {
            return Err(CryptoError::InvalidKey("Secret key must be 32 bytes".to_string()).into());
        }
        
        let signing_key = SigningKey::from_bytes(&secret_bytes.try_into().unwrap());
        
        // Sign the data
        let signature: Signature = signing_key.sign(data);
        
        Ok(hex::encode(signature.to_bytes()))
    }
    
    /// Verify Ed25519 signature with public key
    pub fn verify(&self, data: &[u8], signature: &str, public_key: &str) -> Result<bool> {
        if !self.initialized {
            return Err(CryptoError::InitializationFailed("Crypto not initialized".to_string()).into());
        }
        
        use ed25519_dalek::{VerifyingKey, Signature, Verifier};
        
        // Parse the public key and signature from hex
        let public_bytes = hex::decode(public_key)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid public key hex: {}", e)))?;
        
        let signature_bytes = hex::decode(signature)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid signature hex: {}", e)))?;
        
        if public_bytes.len() != 32 {
            return Err(CryptoError::InvalidKey("Public key must be 32 bytes".to_string()).into());
        }
        
        if signature_bytes.len() != 64 {
            return Err(CryptoError::InvalidKey("Signature must be 64 bytes".to_string()).into());
        }
        
        let verifying_key = VerifyingKey::from_bytes(&public_bytes.try_into().unwrap())
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid public key: {}", e)))?;
        
        let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());
        
        // Verify the signature
        match verifying_key.verify(data, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Generate BLAKE3 hash (Veilid compatible)
    pub fn hash(&self, data: &[u8]) -> Result<String> {
        let hash = blake3::hash(data);
        Ok(hex::encode(hash.as_bytes()))
    }
    
    /// Generate a cryptographic hash using BLAKE3
    pub fn veilid_hash(&self, data: &[u8]) -> Result<String> {
        // Use BLAKE3 as Veilid does
        self.hash(data)
    }
    
    /// Generate a random 32-byte nonce
    pub fn generate_nonce(&self) -> Result<String> {
        use rand::RngCore;
        let mut nonce = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        Ok(hex::encode(nonce))
    }
    
    /// Derive a shared secret using X25519 key exchange
    pub fn derive_shared_secret(&self, our_secret: &str, their_public: &str) -> Result<String> {
        if !self.initialized {
            return Err(CryptoError::InitializationFailed("Crypto not initialized".to_string()).into());
        }
        
        use x25519_dalek::{StaticSecret, PublicKey};
        
        // Parse our secret key from hex
        let secret_bytes = hex::decode(our_secret)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid secret key hex: {}", e)))?;
        
        if secret_bytes.len() != 32 {
            return Err(CryptoError::InvalidKey("Secret key must be 32 bytes".to_string()).into());
        }
        
        // Parse their public key from hex
        let public_bytes = hex::decode(their_public)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid public key hex: {}", e)))?;
            
        if public_bytes.len() != 32 {
            return Err(CryptoError::InvalidKey("Public key must be 32 bytes".to_string()).into());
        }
        
        // Create X25519 keys with explicit array conversion
        let secret_array: [u8; 32] = secret_bytes.try_into().unwrap();
        let public_array: [u8; 32] = public_bytes.try_into().unwrap();
        
        let our_secret_key = StaticSecret::from(secret_array);
        let their_public_key = PublicKey::from(public_array);
        
        // Perform the key exchange
        let shared_secret = our_secret_key.diffie_hellman(&their_public_key);
        
        Ok(hex::encode(shared_secret.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_manager_new() {
        let crypto = CryptoManager::new().unwrap();
        assert!(crypto.initialized);
    }

    #[test]
    fn test_generate_keypair() {
        let crypto = CryptoManager::new().unwrap();
        let (public_key, secret_key) = crypto.generate_keypair().unwrap();
        
        // Keys should be hex-encoded
        assert_eq!(public_key.len(), 64); // 32 bytes * 2 hex chars
        assert_eq!(secret_key.len(), 64); // 32 bytes * 2 hex chars
        
        // Should be valid hex
        hex::decode(&public_key).unwrap();
        hex::decode(&secret_key).unwrap();
    }

    #[test]
    fn test_sign_and_verify() {
        let crypto = CryptoManager::new().unwrap();
        let (public_key, secret_key) = crypto.generate_keypair().unwrap();
        
        let data = b"Hello, Veilid!";
        let signature = crypto.sign(data, &secret_key).unwrap();
        
        // Signature should be 64 bytes hex-encoded
        assert_eq!(signature.len(), 128); // 64 bytes * 2 hex chars
        
        // Should verify correctly
        let is_valid = crypto.verify(data, &signature, &public_key).unwrap();
        assert!(is_valid);
        
        // Should fail with wrong data
        let wrong_data = b"Wrong data";
        let is_valid = crypto.verify(wrong_data, &signature, &public_key).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_hash() {
        let crypto = CryptoManager::new().unwrap();
        let data = b"Hello, BLAKE3!";
        let hash = crypto.hash(data).unwrap();
        
        // BLAKE3 hash should be 32 bytes hex-encoded
        assert_eq!(hash.len(), 64); // 32 bytes * 2 hex chars
        
        // Same data should produce same hash
        let hash2 = crypto.hash(data).unwrap();
        assert_eq!(hash, hash2);
        
        // Different data should produce different hash
        let different_data = b"Different data";
        let hash3 = crypto.hash(different_data).unwrap();
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_veilid_hash() {
        let crypto = CryptoManager::new().unwrap();
        let data = b"Test data";
        
        // veilid_hash should be same as hash for now
        let hash1 = crypto.hash(data).unwrap();
        let hash2 = crypto.veilid_hash(data).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_generate_nonce() {
        let crypto = CryptoManager::new().unwrap();
        let nonce1 = crypto.generate_nonce().unwrap();
        let nonce2 = crypto.generate_nonce().unwrap();
        
        // Nonces should be 32 bytes hex-encoded
        assert_eq!(nonce1.len(), 64); // 32 bytes * 2 hex chars
        assert_eq!(nonce2.len(), 64);
        
        // Should be different
        assert_ne!(nonce1, nonce2);
        
        // Should be valid hex
        hex::decode(&nonce1).unwrap();
        hex::decode(&nonce2).unwrap();
    }

    #[test]
    fn test_generate_x25519_keypair() {
        let crypto = CryptoManager::new().unwrap();
        let (public_key, secret_key) = crypto.generate_x25519_keypair().unwrap();
        
        // Keys should be hex-encoded
        assert_eq!(public_key.len(), 64); // 32 bytes * 2 hex chars
        assert_eq!(secret_key.len(), 64); // 32 bytes * 2 hex chars
        
        // Should be valid hex
        hex::decode(&public_key).unwrap();
        hex::decode(&secret_key).unwrap();
    }

    #[test]
    fn test_derive_shared_secret() {
        let crypto = CryptoManager::new().unwrap();
        
        // Generate two X25519 keypairs
        let (alice_public, alice_secret) = crypto.generate_x25519_keypair().unwrap();
        let (bob_public, bob_secret) = crypto.generate_x25519_keypair().unwrap();
        
        // Derive shared secrets
        let alice_shared = crypto.derive_shared_secret(&alice_secret, &bob_public).unwrap();
        let bob_shared = crypto.derive_shared_secret(&bob_secret, &alice_public).unwrap();
        
        // Both parties should derive the same shared secret
        assert_eq!(alice_shared, bob_shared);
        
        // Shared secret should be 32 bytes hex-encoded
        assert_eq!(alice_shared.len(), 64); // 32 bytes * 2 hex chars
        
        // Should be valid hex
        hex::decode(&alice_shared).unwrap();
    }
} 