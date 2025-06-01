use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

/// Unique identifier for a Veilid app package
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(pub String);

impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AppId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Veilid URI for app discovery and installation  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VeilUri {
    pub scheme: String,
    pub app_id: AppId,
    pub version: Option<String>,
}

impl VeilUri {
    pub fn new(app_id: AppId, version: Option<String>) -> Self {
        Self {
            scheme: "veil".to_string(),
            app_id,
            version,
        }
    }
}

impl fmt::Display for VeilUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.version {
            Some(version) => write!(f, "{}:///app/{}/{}", self.scheme, self.app_id, version),
            None => write!(f, "{}:///app/{}", self.scheme, self.app_id),
        }
    }
}

/// Complete app information including metadata and DHT location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: AppId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub developer: String,
    pub category: String,
    pub size_bytes: u64,
    pub download_count: u64,
    pub rating: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub entry_point: String,
    pub veilid_identity: Option<String>,
    pub signature: Option<String>,
}

impl AppInfo {
    pub fn uri(&self) -> VeilUri {
        VeilUri::new(self.id.clone(), Some(self.version.clone()))
    }

    pub fn uri_latest(&self) -> VeilUri {
        VeilUri::new(self.id.clone(), None)
    }
    
    /// Generate HTTPS access URL for web compatibility
    pub fn access_url(&self) -> String {
        format!("https://www.roselite.app/access/{}/{}", self.id.0, self.version)
    }
    
    /// Generate HTTPS access URL for latest version
    pub fn access_url_latest(&self) -> String {
        format!("https://www.roselite.app/access/{}", self.id.0)
    }
}

/// Search filters for app discovery
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilter {
    pub query: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub developer: Option<String>,
    pub min_rating: Option<f32>,
    pub max_size_bytes: Option<u64>,
    pub sort_by: Option<SortBy>,
    pub limit: Option<usize>,
}

/// Sort options for app listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Date,
    Rating,
    Downloads,
    Developer,
}

/// Installation status for local apps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallStatus {
    NotInstalled,
    Installed { version: String },
    UpdateAvailable { current: String, latest: String },
} 