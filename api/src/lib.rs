//! # WebArcade Plugin API (Lightweight)
//!
//! Minimal API for building WebArcade plugins with fast compile times.
//! This crate provides only what's needed for FFI plugin communication.
//!
//! ## Features
//!
//! - `bridge` - Enable HTTP bridge functionality (tokio, http types). Only needed
//!   for plugins that define routes in their Cargo.toml.
//!
//! ## Quick Start
//!
//! ```rust
//! use api::prelude::*;
//!
//! pub struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn metadata(&self) -> PluginMetadata {
//!         PluginMetadata {
//!             id: "my-plugin".into(),
//!             name: "My Plugin".into(),
//!             version: "1.0.0".into(),
//!             description: "A plugin".into(),
//!             author: "You".into(),
//!             dependencies: vec![],
//!         }
//!     }
//! }
//! ```

// Core modules (always available)
pub mod plugin;

// Bridge modules (only with "bridge" feature)
#[cfg(feature = "bridge")]
pub mod http;
#[cfg(feature = "bridge")]
pub mod ffi_http;

// Re-export core types (always available)
pub use plugin::{Plugin, PluginMetadata};

// Re-export bridge types (only with "bridge" feature)
#[cfg(feature = "bridge")]
pub use http::{HttpRequest, HttpResponse, MultipartField, json_response, error_response};
#[cfg(feature = "bridge")]
pub use ffi_http::{Request as FfiRequest, Response as FfiResponse};

// Backward compatibility aliases (only with "bridge" feature)
#[cfg(feature = "bridge")]
pub use http::HttpRequest as Request;
#[cfg(feature = "bridge")]
pub use http::HttpResponse as Response;

// Re-export dependencies for use in generated code
pub use serde::{Serialize, Deserialize};
pub use serde_json::{self, json, Value};
pub use log;

// Bridge-only re-exports
#[cfg(feature = "bridge")]
pub use base64;
#[cfg(feature = "bridge")]
pub use tokio;
#[cfg(feature = "bridge")]
pub use bytes::Bytes;

// Prelude for convenient imports
pub mod prelude {
    pub use crate::plugin::{Plugin, PluginMetadata};
    pub use serde::{Serialize, Deserialize};
    pub use serde_json::{json, Value};

    // Bridge types in prelude (only with "bridge" feature)
    #[cfg(feature = "bridge")]
    pub use crate::http::{HttpRequest, HttpResponse, MultipartField, json_response, error_response};
    #[cfg(feature = "bridge")]
    pub use crate::ffi_http::{Request as FfiRequest, Response as FfiResponse};
}
