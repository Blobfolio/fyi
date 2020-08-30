/*!
# FYI Menu: SIMD

This module provides an alternative SIMD-optimized version of `KeyMaster`. It
is exposed when building the crate with the `simd` feature. Nightly Rust is
required.
*/

mod keymaster;
pub use keymaster::KeyMaster;
