use pqc_kyber::*;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use sha2::{Sha256, Digest};
use anyhow::{Result, anyhow};
use rand::thread_rng;

#[derive(Clone)]
pub struct KyberKem {
    ciphertext_size: usize,
}

impl KyberKem {
    pub fn new() -> Self {
        Self {
            ciphertext_size: KYBER_CIPHERTEXTBYTES,
        }
    }
    
    pub fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut rng = thread_rng();
        let keys = keypair(&mut rng).map_err(|_| anyhow!("Keypair generation failed"))?;
        Ok((keys.public.to_vec(), keys.secret.to_vec()))
    }
    
    pub fn encapsulate(&self, public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let pk: [u8; KYBER_PUBLICKEYBYTES] = public_key.try_into()
            .map_err(|_| anyhow!("Invalid public key size"))?;
        
        let mut rng = thread_rng();
        let (ciphertext, shared_secret) = encapsulate(&pk, &mut rng)
            .map_err(|_| anyhow!("Encapsulation failed"))?;
            
        Ok((ciphertext.to_vec(), shared_secret.to_vec()))
    }
    
    pub fn decapsulate(&self, ciphertext: &[u8], secret_key: &[u8]) -> Result<Vec<u8>> {
        let ct: [u8; KYBER_CIPHERTEXTBYTES] = ciphertext.try_into()
            .map_err(|_| anyhow!("Invalid ciphertext size"))?;
        let sk: [u8; KYBER_SECRETKEYBYTES] = secret_key.try_into()
            .map_err(|_| anyhow!("Invalid secret key size"))?;
            
        let shared_secret = decapsulate(&ct, &sk)
            .map_err(|_| anyhow!("Decapsulation failed"))?;
            
        Ok(shared_secret.to_vec())
    }
    
    pub fn encrypt_message(&self, message: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        let (ciphertext, shared_secret) = self.encapsulate(public_key)?;
        
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        let aes_key = hasher.finalize();
        
        let key = Key::<Aes256Gcm>::from_slice(&aes_key);
        let cipher = Aes256Gcm::new(key);
        
        let mut rng = thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rand::Rng::fill(&mut rng, &mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_message = cipher.encrypt(nonce, message.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;
        
        let mut result = ciphertext;
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&encrypted_message);
        
        Ok(result)
    }
    
    pub fn decrypt_message(&self, encrypted_data: &[u8], secret_key: &[u8]) -> Result<String> {
        if encrypted_data.len() < self.ciphertext_size + 12 {
            return Err(anyhow!("Encrypted data too short"));
        }
        
        let ciphertext = &encrypted_data[..self.ciphertext_size];
        let nonce_bytes = &encrypted_data[self.ciphertext_size..self.ciphertext_size + 12];
        let encrypted_message = &encrypted_data[self.ciphertext_size + 12..];
        
        let shared_secret = self.decapsulate(ciphertext, secret_key)?;
        
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        let aes_key = hasher.finalize();
        
        let key = Key::<Aes256Gcm>::from_slice(&aes_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let decrypted = cipher.decrypt(nonce, encrypted_message)
            .map_err(|e| anyhow!("Decryption failed: {:?}", e))?;
        
        Ok(String::from_utf8(decrypted)?)
    }
}

impl Default for KyberKem {
    fn default() -> Self {
        Self::new()
    }
}
