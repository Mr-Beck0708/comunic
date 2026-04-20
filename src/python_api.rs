use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyException};
use crate::crypto::{KyberKem, DilithiumSignature};

#[pyclass]
pub struct PyKyberKem {
    inner: KyberKem,
}

#[pymethods]
impl PyKyberKem {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: KyberKem::new(),
        }
    }

    pub fn generate_keypair(&self) -> PyResult<(Vec<u8>, Vec<u8>)> {
        self.inner.generate_keypair().map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn encapsulate(&self, public_key: &[u8]) -> PyResult<(Vec<u8>, Vec<u8>)> {
        self.inner.encapsulate(public_key).map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn decapsulate(&self, ciphertext: &[u8], secret_key: &[u8]) -> PyResult<Vec<u8>> {
        self.inner.decapsulate(ciphertext, secret_key).map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn encrypt_message(&self, message: &str, public_key: &[u8]) -> PyResult<Vec<u8>> {
        self.inner.encrypt_message(message, public_key).map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn decrypt_message(&self, encrypted_data: &[u8], secret_key: &[u8]) -> PyResult<String> {
        self.inner.decrypt_message(encrypted_data, secret_key).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[pyclass]
pub struct PyDilithiumSignature {
    inner: DilithiumSignature,
}

#[pymethods]
impl PyDilithiumSignature {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: DilithiumSignature::new(),
        }
    }

    pub fn generate_keypair(&self) -> PyResult<(Vec<u8>, Vec<u8>)> {
        self.inner.generate_keypair().map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn sign(&self, message: &[u8], signing_key: &[u8]) -> PyResult<Vec<u8>> {
        self.inner.sign(message, signing_key).map_err(|e| PyException::new_err(e.to_string()))
    }

    pub fn verify(&self, message: &[u8], signature: &[u8], verification_key: &[u8]) -> PyResult<bool> {
        self.inner.verify(message, signature, verification_key).map_err(|e| PyException::new_err(e.to_string()))
    }
}
