/*!
# FYI Msg (SIMD)

This module provides an alternative SIMD-optimized version of `Toc`. It is
exposed when building the crate with the `simd` feature. Nightly Rust is
required.
*/

mod toc;
pub use toc::Toc;
