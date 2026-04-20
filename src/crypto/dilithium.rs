use dilithium::{MlDsaKeyPair, ML_DSA_44, DilithiumSignature as SigStruct};
use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct DilithiumSignature;

impl DilithiumSignature {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let kp = MlDsaKeyPair::generate(ML_DSA_44)
            .map_err(|e| anyhow!("Dilithium keypair generation failed: {:?}", e))?;
        // to_bytes() returns [mode_tag | pk | sk], which MlDsaKeyPair::from_bytes() expects.
        Ok((kp.to_bytes().to_vec(), kp.public_key().to_vec()))
    }
    
    pub fn sign(&self, message: &[u8], signing_key: &[u8]) -> Result<Vec<u8>> {
        // MlDsaKeyPair::from_bytes expects the full keypair bytes
        let kp = MlDsaKeyPair::from_bytes(signing_key)
            .map_err(|e| anyhow!("Invalid signing key: {:?}", e))?;
        
        let signature = kp.sign(message, b"")
            .map_err(|e| anyhow!("Signing failed: {:?}", e))?;
            
        Ok(signature.as_bytes().to_vec())
    }
    
    pub fn verify(&self, message: &[u8], signature: &[u8], verification_key: &[u8]) -> Result<bool> {
        let sig_obj = SigStruct::from_bytes(signature.to_vec());
        
        let is_valid = MlDsaKeyPair::verify(
            verification_key,
            &sig_obj,
            message,
            b"",
            ML_DSA_44
        );
        
        Ok(is_valid)
    }
}

impl Default for DilithiumSignature {
    fn default() -> Self {
        Self::new()
    }
}
