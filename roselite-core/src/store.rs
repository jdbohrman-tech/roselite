use crate::{error::*, types::*, package::Package, veilid::VeilidConnection};
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;
use tracing;

/// Main app store interface
#[async_trait]
pub trait AppStore {
    /// Search for apps using filters
    async fn search(&self, filter: &SearchFilter) -> Result<Vec<AppInfo>>;
    
    /// Get app by ID
    async fn get_app(&self, app_id: &AppId) -> Result<Option<AppInfo>>;
    
    /// Get latest version of an app
    async fn get_latest_version(&self, app_id: &AppId) -> Result<Option<String>>;
    
    /// Publish an app package
    async fn publish(&mut self, package: Package) -> Result<VeilUri>;
    
    /// Download a package
    async fn download(&self, uri: &VeilUri) -> Result<Package>;
    
    /// List featured/popular apps
    async fn featured(&self, limit: Option<usize>) -> Result<Vec<AppInfo>>;
}

/// Veilid DHT-based app store implementation
pub struct VeilidStore {
    veilid: VeilidConnection,
}

impl VeilidStore {
    pub async fn new() -> Result<Self> {
        let mut veilid = VeilidConnection::new().await?;
        veilid.connect().await?;
        
        Ok(Self { veilid })
    }
    
    /// Properly shutdown the Veilid connection
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down VeilidStore...");
        self.veilid.disconnect().await?;
        tracing::info!("VeilidStore shutdown complete");
        Ok(())
    }
    
    /// Generate DHT key for app storage using slug
    fn app_key(slug: &str, version: &str) -> String {
        format!("roselite:app:{}:{}", slug, version)
    }
    
    /// Generate DHT key for app metadata using slug
    fn metadata_key(slug: &str) -> String {
        format!("roselite:metadata:{}", slug)
    }
    
    /// Generate DHT key for slug-to-name mapping
    fn slug_mapping_key() -> String {
        "roselite:slug_mapping".to_string()
    }
    
    /// Generate DHT key for name-to-slug mapping
    fn name_mapping_key() -> String {
        "roselite:name_mapping".to_string()
    }
    
    /// Generate DHT key for the app index
    fn index_key() -> String {
        "roselite:index".to_string()
    }
    
    /// Generate DHT key for featured apps
    fn featured_key() -> String {
        "roselite:featured".to_string()
    }
    
    /// Add app to the global index and mapping tables
    async fn add_to_index(&mut self, app_info: &AppInfo) -> Result<()> {
        let index_key = Self::index_key();
        
        // Get current index
        let mut app_index: HashMap<String, AppInfo> = match self.veilid.dht_get(&index_key).await? {
            Some(data) => serde_json::from_slice(&data)
                .map_err(|e| RoseliteError::SerializationError(format!("Failed to deserialize app index: {}", e)))?,
            None => HashMap::new(),
        };
        
        // Add/update app in index (use slug as key)
        app_index.insert(app_info.slug.clone(), app_info.clone());
        
        // Update slug-to-name mapping
        let mut slug_mapping: HashMap<String, String> = match self.veilid.dht_get(&Self::slug_mapping_key()).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => HashMap::new(),
        };
        slug_mapping.insert(app_info.slug.clone(), app_info.name.clone());
        
        // Update name-to-slug mapping
        let mut name_mapping: HashMap<String, String> = match self.veilid.dht_get(&Self::name_mapping_key()).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => HashMap::new(),
        };
        name_mapping.insert(app_info.name.clone(), app_info.slug.clone());
        
        // Serialize and store all updates
        let serialized_index = serde_json::to_vec(&app_index)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize app index: {}", e)))?;
        let serialized_slug_mapping = serde_json::to_vec(&slug_mapping)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize slug mapping: {}", e)))?;
        let serialized_name_mapping = serde_json::to_vec(&name_mapping)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize name mapping: {}", e)))?;
        
        self.veilid.dht_set(&index_key, &serialized_index).await?;
        self.veilid.dht_set(&Self::slug_mapping_key(), &serialized_slug_mapping).await?;
        self.veilid.dht_set(&Self::name_mapping_key(), &serialized_name_mapping).await?;
        
        Ok(())
    }
    
    /// Get all apps from the index
    async fn get_app_index(&self) -> Result<HashMap<String, AppInfo>> {
        let index_key = Self::index_key();
        
        match self.veilid.dht_get(&index_key).await? {
            Some(data) => {
                let app_index: HashMap<String, AppInfo> = serde_json::from_slice(&data)
                    .map_err(|e| RoseliteError::SerializationError(format!("Failed to deserialize app index: {}", e)))?;
                Ok(app_index)
            }
            None => Ok(HashMap::new()),
        }
    }
    
    /// Resolve any identifier (name, slug, or app ID) to a slug
    async fn resolve_to_slug(&self, identifier: &str) -> Result<Option<String>> {
        // First check if it's already a slug by looking in the slug mapping
        let slug_mapping: HashMap<String, String> = match self.veilid.dht_get(&Self::slug_mapping_key()).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => HashMap::new(),
        };
        
        if slug_mapping.contains_key(identifier) {
            return Ok(Some(identifier.to_string()));
        }
        
        // Check if it's a name that maps to a slug
        let name_mapping: HashMap<String, String> = match self.veilid.dht_get(&Self::name_mapping_key()).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => HashMap::new(),
        };
        
        if let Some(slug) = name_mapping.get(identifier) {
            return Ok(Some(slug.clone()));
        }
        
        // If not found in mappings, it might be an app ID, check the index
        let app_index = self.get_app_index().await?;
        for app_info in app_index.values() {
            if app_info.id.0 == identifier {
                return Ok(Some(app_info.slug.clone()));
            }
        }
        
        Ok(None)
    }
}

#[async_trait]
impl AppStore for VeilidStore {
    async fn search(&self, filter: &SearchFilter) -> Result<Vec<AppInfo>> {
        let app_index = self.get_app_index().await?;
        let mut results: Vec<AppInfo> = app_index.into_values().collect();
        
        // Apply query filter
        if let Some(query) = &filter.query {
            let query_lower = query.to_lowercase();
            results.retain(|app| {
                app.name.to_lowercase().contains(&query_lower) ||
                app.description.to_lowercase().contains(&query_lower) ||
                app.developer.to_lowercase().contains(&query_lower) ||
                app.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            });
        }
        
        // Apply category filter
        if let Some(category) = &filter.category {
            results.retain(|app| &app.category == category);
        }
        
        // Apply tag filter
        if !filter.tags.is_empty() {
            results.retain(|app| {
                filter.tags.iter().any(|tag| app.tags.contains(tag))
            });
        }
        
        // Apply developer filter
        if let Some(developer) = &filter.developer {
            results.retain(|app| &app.developer == developer);
        }
        
        // Apply rating filter
        if let Some(min_rating) = filter.min_rating {
            results.retain(|app| app.rating >= min_rating);
        }
        
        // Apply sorting
        if let Some(sort_by) = &filter.sort_by {
            match sort_by {
                SortBy::Name => results.sort_by(|a, b| a.name.cmp(&b.name)),
                SortBy::Date => results.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                SortBy::Rating => results.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal)),
                SortBy::Downloads => results.sort_by(|a, b| b.download_count.cmp(&a.download_count)),
                SortBy::Developer => results.sort_by(|a, b| a.developer.cmp(&b.developer)),
            }
        }
        
        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }
    
    async fn get_app(&self, app_id: &AppId) -> Result<Option<AppInfo>> {
        // Resolve the identifier to a slug
        let slug = match self.resolve_to_slug(&app_id.0).await? {
            Some(slug) => slug,
            None => return Ok(None),
        };
        
        // Look up by slug
        let metadata_key = Self::metadata_key(&slug);
        
        match self.veilid.dht_get(&metadata_key).await? {
            Some(data) => {
                let app_info: AppInfo = serde_json::from_slice(&data)
                    .map_err(|e| RoseliteError::SerializationError(format!("Failed to deserialize app info: {}", e)))?;
                Ok(Some(app_info))
            }
            None => Ok(None),
        }
    }
    
    async fn get_latest_version(&self, app_id: &AppId) -> Result<Option<String>> {
        if let Some(app_info) = self.get_app(app_id).await? {
            Ok(Some(app_info.version))
        } else {
            Ok(None)
        }
    }
    
    async fn publish(&mut self, package: Package) -> Result<VeilUri> {
        // Generate slug if not provided
        let slug = if package.manifest.slug.is_empty() {
            Package::generate_slug(&package.manifest.name)
        } else {
            package.manifest.slug.clone()
        };
        
        // Create app info from package
        let app_info = AppInfo {
            id: AppId(slug.clone()), // Use slug as the app ID
            name: package.manifest.name.clone(),
            slug: slug.clone(),
            version: package.manifest.version.clone(),
            description: package.manifest.description.clone(),
            developer: package.manifest.developer.clone(),
            category: package.manifest.category.clone(),
            size_bytes: package.size_bytes,
            download_count: 0,
            rating: 0.0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: package.manifest.tags.clone(),
            entry_point: package.manifest.entry.clone(),
            veilid_identity: Some(package.manifest.author.clone()),
            signature: Some("placeholder_signature".to_string()),
        };
        
        // Store package data using slug
        let package_data = serde_json::to_vec(&package)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize package: {}", e)))?;
        
        let package_key = Self::app_key(&slug, &app_info.version);
        self.veilid.dht_set(&package_key, &package_data).await?;
        
        // Store app metadata using slug
        let app_data = serde_json::to_vec(&app_info)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize app info: {}", e)))?;
        
        let metadata_key = Self::metadata_key(&slug);
        self.veilid.dht_set(&metadata_key, &app_data).await?;
        
        // Add to global index and mappings
        self.add_to_index(&app_info).await?;
        
        tracing::info!("Published app {} version {} to Veilid DHT", app_info.name, app_info.version);
        
        Ok(VeilUri::new(AppId(slug), Some(app_info.version)))
    }
    
    async fn download(&self, uri: &VeilUri) -> Result<Package> {
        // Resolve the app_id to a slug
        let slug = match self.resolve_to_slug(&uri.app_id.0).await? {
            Some(slug) => slug,
            None => return Err(RoseliteError::AppNotFound(uri.app_id.to_string())),
        };
        
        let package_key = if let Some(version) = &uri.version {
            Self::app_key(&slug, version)
        } else {
            // If no version specified, get latest
            let latest_version = self.get_latest_version(&AppId(slug.clone())).await?
                .ok_or_else(|| RoseliteError::AppNotFound(uri.app_id.to_string()))?;
            Self::app_key(&slug, &latest_version)
        };
        
        match self.veilid.dht_get(&package_key).await? {
            Some(data) => {
                let package: Package = serde_json::from_slice(&data)
                    .map_err(|e| RoseliteError::SerializationError(format!("Failed to deserialize package: {}", e)))?;
                Ok(package)
            }
            None => Err(RoseliteError::AppNotFound(uri.app_id.to_string())),
        }
    }
    
    async fn featured(&self, limit: Option<usize>) -> Result<Vec<AppInfo>> {
        let featured_key = Self::featured_key();
        
        match self.veilid.dht_get(&featured_key).await? {
            Some(data) => {
                let mut featured_apps: Vec<AppInfo> = serde_json::from_slice(&data)
                    .map_err(|e| RoseliteError::SerializationError(format!("Failed to deserialize featured apps: {}", e)))?;
                
                if let Some(limit) = limit {
                    featured_apps.truncate(limit);
                }
                
                Ok(featured_apps)
            }
            None => {
                // Fallback to most downloaded apps if no featured list exists
                let filter = SearchFilter {
                    query: None,
                    category: None,
                    tags: Vec::new(),
                    developer: None,
                    min_rating: None,
                    max_size_bytes: None,
                    sort_by: None,
                    limit: Some(10),
                };
                self.search(&filter).await
            }
        }
    }
} 