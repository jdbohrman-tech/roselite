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
    
    /// Generate DHT key for app storage
    fn app_key(app_id: &AppId, version: &str) -> String {
        format!("roselite:app:{}:{}", app_id, version)
    }
    
    /// Generate DHT key for app metadata
    fn metadata_key(app_id: &AppId) -> String {
        format!("roselite:metadata:{}", app_id)
    }
    
    /// Generate DHT key for the app index
    fn index_key() -> String {
        "roselite:index".to_string()
    }
    
    /// Generate DHT key for featured apps
    fn featured_key() -> String {
        "roselite:featured".to_string()
    }
    
    /// Add app to the global index
    async fn add_to_index(&mut self, app_info: &AppInfo) -> Result<()> {
        let index_key = Self::index_key();
        
        // Get current index
        let mut app_index: HashMap<String, AppInfo> = match self.veilid.dht_get(&index_key).await? {
            Some(data) => serde_json::from_slice(&data)
                .map_err(|e| RoseliteError::SerializationError(format!("Failed to deserialize app index: {}", e)))?,
            None => HashMap::new(),
        };
        
        // Add/update app in index
        app_index.insert(app_info.id.0.clone(), app_info.clone());
        
        // Serialize and store updated index
        let serialized = serde_json::to_vec(&app_index)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize app index: {}", e)))?;
        
        self.veilid.dht_set(&index_key, &serialized).await?;
        
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
        let metadata_key = Self::metadata_key(app_id);
        
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
        // Create app info from package
        let app_info = AppInfo {
            id: AppId(package.manifest.name.clone()),
            name: package.manifest.name.clone(),
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
        
        // Store package data
        let package_data = serde_json::to_vec(&package)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize package: {}", e)))?;
        
        let package_key = Self::app_key(&app_info.id, &app_info.version);
        self.veilid.dht_set(&package_key, &package_data).await?;
        
        // Store app metadata
        let app_data = serde_json::to_vec(&app_info)
            .map_err(|e| RoseliteError::SerializationError(format!("Failed to serialize app info: {}", e)))?;
        
        let metadata_key = Self::metadata_key(&app_info.id);
        self.veilid.dht_set(&metadata_key, &app_data).await?;
        
        // Add to global index
        self.add_to_index(&app_info).await?;
        
        tracing::info!("Published app {} version {} to Veilid DHT", app_info.id, app_info.version);
        
        Ok(VeilUri::new(app_info.id, Some(app_info.version)))
    }
    
    async fn download(&self, uri: &VeilUri) -> Result<Package> {
        let package_key = if let Some(version) = &uri.version {
            Self::app_key(&uri.app_id, version)
        } else {
            // If no version specified, get latest
            let latest_version = self.get_latest_version(&uri.app_id).await?
                .ok_or_else(|| RoseliteError::AppNotFound(uri.app_id.to_string()))?;
            Self::app_key(&uri.app_id, &latest_version)
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