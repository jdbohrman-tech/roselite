//! Roselite Core Library
//! 
//! Core functionality for the Roselite decentralized app store built on Veilid.
//! This library provides:
//! - Package format (.veilidpkg) handling
//! - Veilid DHT integration for app storage/discovery
//! - Cryptographic signing and verification
//! - App metadata management

pub mod error;
pub mod package;
pub mod store;
pub mod crypto;
pub mod types;
pub mod veilid;

// Re-export commonly used types
pub use error::{Result, RoseliteError};
pub use package::{Package, PackageBuilder, PackageManifest};
pub use store::{AppStore, VeilidStore};
pub use types::{AppId, AppInfo, VeilUri};

/// Current version of the Roselite package format
pub const PACKAGE_FORMAT_VERSION: &str = "1.0.0";

/// File extension for Veilid packages
pub const PACKAGE_EXTENSION: &str = ".veilidpkg";

/// Manifest filename within packages
pub const MANIFEST_FILENAME: &str = "veilid.json"; 