use wasm_bindgen::prelude::*;
use crate::crypto::{KyberKem, DilithiumSignature};

#[wasm_bindgen]
pub struct WasmKyberKem {
    inner: KyberKem,
}

#[wasm_bindgen]
impl WasmKyberKem {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: KyberKem::new(),
        }
    }

    pub fn generate_keypair(&self) -> Result<JsValue, JsValue> {
        let (pk, sk) = self.inner.generate_keypair()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let array = js_sys::Array::new();
        array.push(&Uint8Array::from(&pk[..]).into());
        array.push(&Uint8Array::from(&sk[..]).into());
        Ok(array.into())
    }

    pub fn encapsulate(&self, public_key: &[u8]) -> Result<JsValue, JsValue> {
        let (ct, ss) = self.inner.encapsulate(public_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let array = js_sys::Array::new();
        array.push(&Uint8Array::from(&ct[..]).into());
        array.push(&Uint8Array::from(&ss[..]).into());
        Ok(array.into())
    }

    pub fn decapsulate(&self, ciphertext: &[u8], secret_key: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.inner.decapsulate(ciphertext, secret_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn encrypt_message(&self, message: &str, public_key: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.inner.encrypt_message(message, public_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn decrypt_message(&self, encrypted_data: &[u8], secret_key: &[u8]) -> Result<String, JsValue> {
        self.inner.decrypt_message(encrypted_data, secret_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub struct WasmDilithiumSignature {
    inner: DilithiumSignature,
}

#[wasm_bindgen]
impl WasmDilithiumSignature {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: DilithiumSignature::new(),
        }
    }

    pub fn generate_keypair(&self) -> Result<JsValue, JsValue> {
        let (sk, pk) = self.inner.generate_keypair()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let array = js_sys::Array::new();
        array.push(&Uint8Array::from(&sk[..]).into());
        array.push(&Uint8Array::from(&pk[..]).into());
        Ok(array.into())
    }

    pub fn sign(&self, message: &[u8], signing_key: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.inner.sign(message, signing_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn verify(&self, message: &[u8], signature: &[u8], verification_key: &[u8]) -> Result<bool, JsValue> {
        self.inner.verify(message, signature, verification_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

use js_sys::Uint8Array;
