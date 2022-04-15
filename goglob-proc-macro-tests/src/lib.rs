//! # Do not use this crate!
//!
//! See the `goglob` crate instead.
//!
//! (This crate facilitates testing `goglob`'s procedural macros with
//! `cargo test` and `trybuild`. It offers no functionality to the end
//! user)

pub fn stub_add(a: usize, b: usize) -> usize {
    a + b
}
