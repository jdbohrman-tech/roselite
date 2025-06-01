use thiserror::Error;

/// Result type alias for Roselite operations
pub type Result<T> = std::result::Result<T, RoseliteError>;

/// Main error type for Roselite operations
#[derive(Error, Debug)]
pub enum RoseliteError {
    #[error("Package error: {0}")]
    Package(#[from] PackageError),

    #[error("Veilid error: {0}")]
    Veilid(#[from] VeilidError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("App not found: {0}")]
    AppNotFound(String),

    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Version mismatch: {0}")]
    VersionMismatch(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Package-specific errors
#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Invalid package format")]
    InvalidFormat,

    #[error("Missing manifest file")]
    MissingManifest,

    #[error("Invalid manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("Package signature verification failed")]
    InvalidSignature,

    #[error("Unsupported package version: {version}")]
    UnsupportedVersion { version: String },

    #[error("Package already exists: {name}")]
    AlreadyExists { name: String },
}

/// Veilid-specific errors
#[derive(Error, Debug)]
pub enum VeilidError {
    #[error("Failed to connect to Veilid network")]
    ConnectionFailed,

    #[error("DHT operation failed: {operation}")]
    DhtOperationFailed { operation: String },

    #[error("App not found in DHT: {app_id}")]
    AppNotFound { app_id: String },

    #[error("Invalid Veilid URI: {uri}")]
    InvalidUri { uri: String },
}

/// Cryptographic errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key format")]
    InvalidKeyFormat,

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Signature generation failed")]
    SignatureGenerationFailed,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Hash computation failed")]
    HashComputationFailed,

    #[error("Crypto initialization failed: {0}")]
    InitializationFailed(String),
} 