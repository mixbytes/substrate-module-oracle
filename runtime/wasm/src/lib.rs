//! The Substrate node template runtime reexported for WebAssembly compile.

#![cfg_attr(not(feature = "std"), no_std)]

pub use oracle_node_runtime::*;
