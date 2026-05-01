pub mod crypto;

#[cfg(not(target_arch = "wasm32"))]
pub mod network;

#[cfg(not(target_arch = "wasm32"))]
pub mod drone;

#[cfg(not(target_arch = "wasm32"))]
pub mod device;

#[cfg(feature = "python")]
pub mod python_api;

#[cfg(target_arch = "wasm32")]
pub mod wasm_api;

pub const VERSION: &str = "0.1.0";
pub const MAX_DEVICES: usize = 5;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn drone_crypto(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<python_api::PyKyberKem>()?;
    m.add_class::<python_api::PyDilithiumSignature>()?;
    Ok(())
}
