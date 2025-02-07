//! Interact with git revisions by parsing them from rev-specs and turning them into rev-specs.
//!
//! One can also describe revisions using a different algorithm.
//! ## Feature Flags
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
#![forbid(unsafe_code)]
#![deny(missing_docs, rust_2018_idioms)]

/// Access to collections optimized for keys that are already a hash.
pub use hash_hasher;

///
pub mod describe;
pub use describe::function::describe;

///
pub mod spec;
