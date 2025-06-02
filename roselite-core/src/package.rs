use serde::{Deserialize, Serialize};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Archive, Builder};
use std::path::Path;
use crate::error::*;
use crate::types::*;
use crate::crypto::CryptoManager;
use std::io::{Read, Write};
use chrono::{DateTime, Utc};
use std::io::Cursor;

/// Package manifest structure (veilid.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub developer: String,
    pub author: String,
    pub category: String,
    pub entry: String,
    pub tags: Vec<String>,
    /// Human-readable URL-safe identifier (auto-generated from name if not provided)
    #[serde(default)]
    pub slug: String,
    pub identity: String,
    pub signature: String,
    pub format_version: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<Permission>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    /// Public key for signature verification
    #[serde(default)]
    pub public_key: String,
}

impl PackageManifest {
    /// Generates a URL-safe slug from the app name
    pub fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter_map(|c| {
                if c.is_alphanumeric() {
                    Some(c)
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    Some('-')
                } else {
                    None
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
    
    /// Ensure the slug is set, generating it from name if needed
    pub fn ensure_slug(&mut self) {
        if self.slug.is_empty() {
            self.slug = Self::generate_slug(&self.name);
        }
    }
}

/// App permissions for sandboxing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Network,
    FileSystem,
    Camera,
    Microphone,
    Clipboard,
}

/// Complete package with manifest and content
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Package {
    pub manifest: PackageManifest,
    pub content: Vec<u8>,
    pub size_bytes: u64,
    pub data: Vec<u8>,
}

impl Package {
    /// Generates a URL-safe slug from the app name
    pub fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter_map(|c| {
                if c.is_alphanumeric() {
                    Some(c)
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    Some('-')
                } else {
                    None
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    /// Load package from .veilidpkg file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read(path).await?;
        Self::from_bytes(content).await
    }

    /// Load package from bytes
    pub async fn from_bytes(content: Vec<u8>) -> Result<Self> {
        let size_bytes = content.len() as u64;
        
        // Decompress the package
        let decoder = GzDecoder::new(&content[..]);
        let mut archive = Archive::new(decoder);
        
        // Find and read the manifest
        let mut manifest_content = Vec::new();
        let mut found_manifest = false;
        
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            
            if path.file_name().and_then(|n| n.to_str()) == Some(crate::MANIFEST_FILENAME) {
                entry.read_to_end(&mut manifest_content)?;
                found_manifest = true;
                break;
            }
        }
        
        if !found_manifest {
            return Err(PackageError::MissingManifest.into());
        }
        
        // Parse manifest
        let manifest: PackageManifest = serde_json::from_slice(&manifest_content)?;
        
        // Validate manifest
        Self::validate_manifest(&manifest)?;
        
        Ok(Package {
            manifest,
            content,
            size_bytes,
            data: Vec::new(),
        })
    }

    /// Convert to app info for listings
    pub fn to_app_info(&self) -> AppInfo {
        let now = Utc::now();
        let identity = self.manifest.identity.clone();
        
        AppInfo {
            id: AppId(identity.clone()),
            name: self.manifest.name.clone(),
            slug: if self.manifest.slug.is_empty() { 
                Self::generate_slug(&self.manifest.name) 
            } else { 
                self.manifest.slug.clone() 
            },
            version: self.manifest.version.clone(),
            description: self.manifest.description.clone(),
            developer: self.manifest.developer.clone(),
            category: self.manifest.category.clone(),
            size_bytes: self.content.len() as u64,
            download_count: 0,
            rating: 0.0,
            created_at: now,
            updated_at: now,
            tags: self.manifest.tags.clone(),
            entry_point: self.manifest.entry.clone(),
            veilid_identity: Some(identity),
            signature: None,
        }
    }

    /// Validate package signature using crypto manager
    pub fn verify_signature(&self, crypto: &CryptoManager) -> Result<bool> {
        if self.manifest.signature.is_empty() || self.manifest.public_key.is_empty() {
            return Ok(false);
        }
        
        // Create the signed data (manifest without signature)
        let mut unsigned_manifest = self.manifest.clone();
        unsigned_manifest.signature = String::new();
        
        let manifest_data = serde_json::to_vec(&unsigned_manifest)
            .map_err(|e| CryptoError::InvalidKey(format!("Failed to serialize manifest: {}", e)))?;
        
        // Verify the signature
        crypto.verify(
            &manifest_data,
            &self.manifest.signature,
            &self.manifest.public_key
        )
    }

    fn validate_manifest(manifest: &PackageManifest) -> Result<()> {
        if manifest.name.is_empty() {
            return Err(PackageError::InvalidManifest { 
                reason: "name cannot be empty".to_string() 
            }.into());
        }
        
        if manifest.version.is_empty() {
            return Err(PackageError::InvalidManifest { 
                reason: "version cannot be empty".to_string() 
            }.into());
        }
        
        if manifest.entry.is_empty() {
            return Err(PackageError::InvalidManifest { 
                reason: "entry point cannot be empty".to_string() 
            }.into());
        }
        
        if manifest.identity.is_empty() {
            return Err(PackageError::InvalidManifest { 
                reason: "identity cannot be empty".to_string() 
            }.into());
        }
        
        Ok(())
    }

    /// Extract individual files from the package for direct serving
    pub async fn extract_files(&self) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        let mut files = std::collections::HashMap::new();
        
        // Decompress the package
        let decoder = GzDecoder::new(Cursor::new(&self.content));
        let mut archive = Archive::new(decoder);
        
        // Extract all files
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            
            // Skip the manifest file since it's metadata
            if path.file_name().and_then(|n| n.to_str()) == Some(crate::MANIFEST_FILENAME) {
                continue;
            }
            
            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;
            
            // Use forward slashes for web compatibility
            let web_path = path.to_string_lossy().replace('\\', "/");
            files.insert(web_path, content);
        }
        
        Ok(files)
    }

    /// Get the entry point file content directly
    pub async fn get_entry_file(&self) -> Result<Vec<u8>> {
        let files = self.extract_files().await?;
        
        files.get(&self.manifest.entry)
            .cloned()
            .ok_or_else(|| PackageError::InvalidManifest {
                reason: format!("Entry file '{}' not found in package", self.manifest.entry)
            }.into())
    }

    /// Get a specific file from the package
    pub async fn get_file(&self, path: &str) -> Result<Option<Vec<u8>>> {
        let files = self.extract_files().await?;
        Ok(files.get(path).cloned())
    }

    /// List all files in the package
    pub async fn list_files(&self) -> Result<Vec<String>> {
        let files = self.extract_files().await?;
        Ok(files.keys().cloned().collect())
    }
}

/// Builder for creating packages
pub struct PackageBuilder {
    name: String,
    version: String,
    description: String,
    developer: String,
    entry: String,
    tags: Vec<String>,
    source_dir: std::path::PathBuf,
    slug: Option<String>,
    identity: Option<String>,
    private_key: Option<String>,
    public_key: Option<String>,
}

impl PackageBuilder {
    pub fn new<P: AsRef<Path>>(name: String, source_dir: P) -> Self {
        Self {
            name,
            version: "1.0.0".to_string(),
            description: String::new(),
            developer: String::new(),
            entry: "index.html".to_string(),
            tags: Vec::new(),
            source_dir: source_dir.as_ref().to_path_buf(),
            slug: None,
            identity: None,
            private_key: None,
            public_key: None,
        }
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn developer(mut self, developer: String) -> Self {
        self.developer = developer;
        self
    }

    pub fn entry(mut self, entry: String) -> Self {
        self.entry = entry;
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn slug(mut self, slug: String) -> Self {
        self.slug = Some(slug);
        self
    }

    pub fn identity(mut self, identity: String) -> Self {
        self.identity = Some(identity);
        self
    }
    
    pub fn keypair(mut self, public_key: String, private_key: String) -> Self {
        self.public_key = Some(public_key);
        self.private_key = Some(private_key);
        self
    }

    /// Build the package with proper signing and compression
    pub async fn build(self) -> Result<Package> {
        let crypto = CryptoManager::new()?;
        
        // Generate or use provided keypair
        let (public_key, private_key) = if let (Some(pub_key), Some(priv_key)) = (self.public_key.clone(), self.private_key.clone()) {
            (pub_key, priv_key)
        } else {
            crypto.generate_keypair()?
        };
        
        let identity = self.identity.clone().unwrap_or_else(|| {
            // Use public key as identity for Veilid compatibility
            public_key.clone()
        });

        let now = Utc::now();
        let mut manifest = PackageManifest {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            developer: self.developer.clone(),
            author: self.developer.clone(),
            category: "general".to_string(),
            entry: self.entry.clone(),
            tags: self.tags.clone(),
            identity: identity.clone(),
            signature: String::new(), // Will be filled after signing
            format_version: crate::PACKAGE_FORMAT_VERSION.to_string(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
            created_at: now,
            updated_at: now,
            public_key: public_key.clone(),
            slug: String::new(),
        };

        // Create tarball from source directory
        let mut tar_data = Vec::new();
        {
            let encoder = GzEncoder::new(&mut tar_data, Compression::default());
            let mut tar_builder = Builder::new(encoder);

            // Add all files from source directory
            if self.source_dir.exists() {
                Self::add_directory_to_tar(&mut tar_builder, &self.source_dir, &self.source_dir).await?;
            }

            // Add manifest to the tar
            let manifest_json = serde_json::to_vec(&manifest)
                .map_err(|e| PackageError::InvalidManifest { 
                    reason: format!("Failed to serialize manifest: {}", e) 
                })?;
            
            let mut header = tar::Header::new_gnu();
            header.set_path(crate::MANIFEST_FILENAME)?;
            header.set_size(manifest_json.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            
            tar_builder.append(&header, manifest_json.as_slice())?;
            
            // Finish the tar
            let encoder = tar_builder.into_inner()?;
            encoder.finish()?;
        }

        // Sign the manifest
        let manifest_data = serde_json::to_vec(&manifest)
            .map_err(|e| CryptoError::InvalidKey(format!("Failed to serialize manifest for signing: {}", e)))?;
        
        let signature = crypto.sign(&manifest_data, &private_key)?;
        manifest.signature = signature;

        let size_bytes = tar_data.len() as u64;
        
        Ok(Package {
            manifest,
            content: tar_data.clone(),
            size_bytes,
            data: tar_data,
        })
    }
    
    /// Add directory contents to tar using walkdir for simplicity
    async fn add_directory_to_tar<W: Write>(
        tar_builder: &mut Builder<W>,
        dir_path: &Path,
        base_path: &Path,
    ) -> Result<()> {
        // Collect all files first to avoid async recursion
        let mut files_to_add = Vec::new();
        Self::collect_files_recursive(dir_path, base_path, &mut files_to_add)?;
        
        // Add all collected files to tar
        for (relative_path, full_path) in files_to_add {
            let file_data = tokio::fs::read(&full_path).await?;
            
            let mut header = tar::Header::new_gnu();
            header.set_path(&relative_path)?;
            header.set_size(file_data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            
            tar_builder.append(&header, file_data.as_slice())?;
        }
        
        Ok(())
    }
    
    /// Recursively collect all files (synchronous)
    fn collect_files_recursive(
        dir_path: &Path,
        base_path: &Path,
        files: &mut Vec<(String, std::path::PathBuf)>,
    ) -> Result<()> {
        use std::fs;
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Recursively collect from subdirectory
                Self::collect_files_recursive(&entry_path, base_path, files)?;
            } else {
                // Add file to collection
                let relative_path = entry_path.strip_prefix(base_path)
                    .map_err(|_| PackageError::InvalidFormat)?;
                
                files.push((
                    relative_path.to_string_lossy().to_string(),
                    entry_path,
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_package_builder() {
        // Create a temporary directory with some test files
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("test_app");
        fs::create_dir_all(&source_dir).unwrap();
        
        // Create test files
        fs::write(source_dir.join("index.html"), b"<html><body>Hello World</body></html>").unwrap();
        fs::write(source_dir.join("app.js"), b"console.log('Hello from JS');").unwrap();
        
        // Create subdirectory with file
        let sub_dir = source_dir.join("assets");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("style.css"), b"body { color: blue; }").unwrap();

        // Build package
        let package = PackageBuilder::new("test-app".to_string(), &source_dir)
            .version("1.0.0".to_string())
            .description("A test application".to_string())
            .developer("Test Developer".to_string())
            .entry("index.html".to_string())
            .tags(vec!["test".to_string(), "demo".to_string()])
            .build()
            .await
            .unwrap();

        // Verify package structure
        assert_eq!(package.manifest.name, "test-app");
        assert_eq!(package.manifest.version, "1.0.0");
        assert_eq!(package.manifest.description, "A test application");
        assert_eq!(package.manifest.developer, "Test Developer");
        assert_eq!(package.manifest.entry, "index.html");
        assert_eq!(package.manifest.tags, vec!["test", "demo"]);
        assert!(!package.manifest.signature.is_empty());
        assert!(!package.manifest.public_key.is_empty());
        assert!(package.size_bytes > 0);
        assert!(!package.content.is_empty());

        // Verify signature
        let crypto = CryptoManager::new().unwrap();
        let is_valid = package.verify_signature(&crypto).unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_package_verification() {
        let crypto = CryptoManager::new().unwrap();
        let (public_key, private_key) = crypto.generate_keypair().unwrap();
        
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("test_app");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("index.html"), b"<html></html>").unwrap();

        // Build package with specific keypair
        let package = PackageBuilder::new("test-app".to_string(), &source_dir)
            .keypair(public_key.clone(), private_key.clone())
            .build()
            .await
            .unwrap();

        // Verify signature
        let is_valid = package.verify_signature(&crypto).unwrap();
        assert!(is_valid);
        
        // Test with empty signature
        let mut package_copy = package.clone();
        package_copy.manifest.signature = String::new();
        let is_valid = package_copy.verify_signature(&crypto).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_manifest_validation() {
        // Test valid manifest
        let valid_manifest = PackageManifest {
            name: "test-app".to_string(),
            version: "1.0.0".to_string(),
            description: "Test app".to_string(),
            developer: "Developer".to_string(),
            author: "Author".to_string(),
            category: "general".to_string(),
            entry: "index.html".to_string(),
            tags: vec![],
            identity: "test-identity".to_string(),
            signature: "test-signature".to_string(),
            format_version: "1.0.0".to_string(),
            dependencies: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            public_key: "test-key".to_string(),
            slug: String::new(),
        };
        
        assert!(Package::validate_manifest(&valid_manifest).is_ok());
        
        // Test invalid manifest (empty name)
        let mut invalid_manifest = valid_manifest.clone();
        invalid_manifest.name = String::new();
        assert!(Package::validate_manifest(&invalid_manifest).is_err());
        
        // Test invalid manifest (empty version)
        let mut invalid_manifest = valid_manifest.clone();
        invalid_manifest.version = String::new();
        assert!(Package::validate_manifest(&invalid_manifest).is_err());
        
        // Test invalid manifest (empty entry)
        let mut invalid_manifest = valid_manifest.clone();
        invalid_manifest.entry = String::new();
        assert!(Package::validate_manifest(&invalid_manifest).is_err());
        
        // Test invalid manifest (empty identity)
        let mut invalid_manifest = valid_manifest;
        invalid_manifest.identity = String::new();
        assert!(Package::validate_manifest(&invalid_manifest).is_err());
    }
} 