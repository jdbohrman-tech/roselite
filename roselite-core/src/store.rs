use crate::{Result, RoseliteError};
use crate::{veilid::VeilidConnection, types::{AppId, VeilUri, AppInfo}, package::Package};
use serde_json;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// High-level abstraction over Veilid DHT for storing Roselite apps.
#[async_trait]
pub trait AppStore {
    /// Publish a package and return both the URI and the updated package with DHT key set
    async fn publish(&mut self, package: Package) -> Result<(VeilUri, Package)>;
    async fn get_app(&self, app_id: &AppId) -> Result<Option<AppInfo>>;
    async fn download(&self, uri: &VeilUri) -> Result<Package>;
    async fn shutdown(&mut self) -> Result<()>;
}

/// Reference to a package record chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRecord {
    /// DHT key for this package record
    pub record_key: String,
    /// Number of subkeys (chunks) in this record
    pub chunk_count: usize,
    /// Size in bytes of this record's content
    pub size_bytes: usize,
}

/// Lookup record that contains metadata and package record references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupRecord {
    /// App metadata
    pub app_info: AppInfo,
    /// List of package records containing the actual package data
    pub package_records: Vec<PackageRecord>,
    /// Total size across all package records
    pub total_size_bytes: usize,
    /// Schema version for future compatibility
    pub schema_version: String,
}

/// Concrete implementation that talks directly to a local Veilid node.
pub struct VeilidStore {
    conn: VeilidConnection,
}

impl VeilidStore {
    /// Create a new store and connect to the Veilid network.
    pub async fn new() -> Result<Self> {
        let mut conn = VeilidConnection::new().await?;
        conn.connect().await?;
        Ok(Self { conn })
    }

    /// Maximum size for a single DHT record (leaving room for metadata)
    const MAX_RECORD_SIZE: usize = 950_000; // ~950KB to stay well under 1MiB
    /// Size per chunk within a record
    const CHUNK_SIZE: usize = 8000; // 8KB chunks for good distribution
}

#[async_trait]
impl AppStore for VeilidStore {
    /// Publish a package into the Veilid DHT using multi-record approach.
    async fn publish(&mut self, package: Package) -> Result<(VeilUri, Package)> {
        let content = &package.content;
        let mut package_records = Vec::new();
        let mut content_offset = 0;

        // Split content across multiple package records if needed
        while content_offset < content.len() {
            // Calculate how much content to put in this record
            let remaining_content = content.len() - content_offset;
            let record_content_size = std::cmp::min(remaining_content, Self::MAX_RECORD_SIZE);
            let record_end = content_offset + record_content_size;
            let record_content = &content[content_offset..record_end];

            // Split this record's content into chunks
            let mut chunks = Vec::new();
            let mut chunk_offset = 0;
            while chunk_offset < record_content.len() {
                let chunk_end = std::cmp::min(chunk_offset + Self::CHUNK_SIZE, record_content.len());
                chunks.push(&record_content[chunk_offset..chunk_end]);
                chunk_offset = chunk_end;
            }

            // Create DHT record for this chunk group
            let record_key = self.conn.create_dht_record_with_cols(chunks.len()).await?;

            // Store chunks in this record
            for (idx, chunk) in chunks.iter().enumerate() {
                self.conn.dht_set_subkey(&record_key, idx as u32, chunk).await?;
            }

            // Track this package record
            package_records.push(PackageRecord {
                record_key: record_key.clone(),
                chunk_count: chunks.len(),
                size_bytes: record_content.len(),
            });

            tracing::info!("Created package record {} with {} chunks ({} bytes)", 
                record_key, chunks.len(), record_content.len());

            content_offset = record_end;
        }

        // Create the lookup record
        let lookup_key = self.conn.create_dht_record_with_cols(1).await?;
        
        // Build app info with the lookup key as the ID
        let mut app_info = package.to_app_info();
        app_info.id = AppId(lookup_key.clone());

        // Create lookup record
        let lookup_record = LookupRecord {
            app_info: app_info.clone(),
            package_records,
            total_size_bytes: content.len(),
            schema_version: "1.0".to_string(),
        };

        // Store lookup record metadata
        let lookup_json = serde_json::to_vec(&lookup_record)?;
        if lookup_json.len() > 1_000_000 { // ~1MB check
            return Err(RoseliteError::ValidationError(
                "Lookup record metadata exceeds 1MB limit".to_string()
            ));
        }

        self.conn.dht_set_subkey(&lookup_key, 0, &lookup_json).await?;

        tracing::info!("Published package with {} package records, lookup key: {}", 
            lookup_record.package_records.len(), lookup_key);

        // Inspect the lookup record (best-effort)
        let _ = self.conn.inspect_record(&lookup_key).await;

        // Update the package with the DHT key
        let mut updated_package = package;
        updated_package.set_dht_key(lookup_key.clone());

        Ok((app_info.uri(), updated_package))
    }

    /// Retrieve application metadata from lookup record.
    async fn get_app(&self, app_id: &AppId) -> Result<Option<AppInfo>> {
        match self.conn.dht_get_subkey(&app_id.0, 0).await? {
            Some(bytes) => {
                // Try to parse as lookup record first
                if let Ok(lookup_record) = serde_json::from_slice::<LookupRecord>(&bytes) {
                    Ok(Some(lookup_record.app_info))
                } else {
                    // Fallback: try to parse as legacy AppInfo for backwards compatibility
                    match serde_json::from_slice::<AppInfo>(&bytes) {
                        Ok(info) => Ok(Some(info)),
                        Err(_) => Ok(None),
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Download the raw package bytes using multi-record approach.
    async fn download(&self, uri: &VeilUri) -> Result<Package> {
        // Get lookup record
        let lookup_bytes = self.conn.dht_get_subkey(&uri.app_id.0, 0).await?
            .ok_or_else(|| RoseliteError::Veilid(crate::error::VeilidError::AppNotFound { 
                app_id: uri.app_id.0.clone() 
            }))?;

        let lookup_record: LookupRecord = serde_json::from_slice(&lookup_bytes)
            .map_err(|_| RoseliteError::ValidationError(
                "Invalid lookup record format".to_string()
            ))?;

        // Download content from all package records
        let mut full_content = Vec::with_capacity(lookup_record.total_size_bytes);
        
        for package_record in &lookup_record.package_records {
            // Download all chunks from this package record
            for subkey in 0..package_record.chunk_count {
                let chunk = self.conn.dht_get_subkey(&package_record.record_key, subkey as u32).await?
                    .ok_or_else(|| RoseliteError::Veilid(crate::error::VeilidError::AppNotFound { 
                        app_id: package_record.record_key.clone() 
                    }))?;
                full_content.extend_from_slice(&chunk);
            }
        }

        // Verify total size matches expectation
        if full_content.len() != lookup_record.total_size_bytes {
            return Err(RoseliteError::ValidationError(format!(
                "Downloaded content size ({} bytes) doesn't match expected size ({} bytes)",
                full_content.len(), lookup_record.total_size_bytes
            )));
        }

        tracing::info!("Downloaded package from {} package records ({} total bytes)", 
            lookup_record.package_records.len(), full_content.len());

        let package = Package::from_bytes(full_content).await?;
        Ok(package)
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.conn.disconnect().await
    }
} 