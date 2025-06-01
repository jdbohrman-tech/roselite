use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use color_eyre::Result;
use roselite_core::types::{AppInfo, AppId, VeilUri};

/// Local app installation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAppInfo {
    pub app_info: AppInfo,
    pub install_path: PathBuf,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub executable_path: PathBuf,
}

/// Local app registry for managing installed apps
pub struct LocalRegistry {
    registry_path: PathBuf,
    apps_dir: PathBuf,
}

impl LocalRegistry {
    /// Create a new local registry
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Unable to find config directory"))?
            .join("roselite");
        
        let apps_dir = dirs::data_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Unable to find data directory"))?
            .join("roselite")
            .join("apps");
        
        Ok(Self {
            registry_path: config_dir.join("installed_apps.json"),
            apps_dir,
        })
    }

    /// Load the installed apps registry
    pub async fn load(&self) -> Result<HashMap<String, LocalAppInfo>> {
        // Ensure config directory exists
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if !self.registry_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.registry_path).await?;
        let registry: HashMap<String, LocalAppInfo> = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// Save the installed apps registry
    pub async fn save(&self, registry: &HashMap<String, LocalAppInfo>) -> Result<()> {
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(registry)?;
        fs::write(&self.registry_path, content).await?;
        Ok(())
    }

    /// Add an app to the registry
    pub async fn add_app(&self, app_info: AppInfo, install_path: PathBuf) -> Result<()> {
        let mut registry = self.load().await?;
        
        let executable_path = install_path.join(&app_info.entry_point);
        
        let local_info = LocalAppInfo {
            app_info: app_info.clone(),
            install_path,
            installed_at: chrono::Utc::now(),
            executable_path,
        };
        
        registry.insert(app_info.id.0.clone(), local_info);
        self.save(&registry).await?;
        Ok(())
    }

    /// Remove an app from the registry
    pub async fn remove_app(&self, app_id: &AppId) -> Result<Option<LocalAppInfo>> {
        let mut registry = self.load().await?;
        let removed = registry.remove(&app_id.0);
        
        if removed.is_some() {
            self.save(&registry).await?;
        }
        
        Ok(removed)
    }

    /// Get an app from the registry
    pub async fn get_app(&self, app_id: &AppId) -> Result<Option<LocalAppInfo>> {
        let registry = self.load().await?;
        Ok(registry.get(&app_id.0).cloned())
    }

    /// List all installed apps
    pub async fn list_apps(&self) -> Result<Vec<LocalAppInfo>> {
        let registry = self.load().await?;
        Ok(registry.into_values().collect())
    }

    /// Get the apps directory path
    pub fn apps_dir(&self) -> &Path {
        &self.apps_dir
    }

    /// Find app by name (fuzzy matching)
    pub async fn find_app_by_name(&self, name: &str) -> Result<Option<LocalAppInfo>> {
        let registry = self.load().await?;
        
        // First try exact match
        if let Some(app) = registry.values().find(|app| app.app_info.name == name) {
            return Ok(Some(app.clone()));
        }
        
        // Then try case-insensitive match
        let name_lower = name.to_lowercase();
        if let Some(app) = registry.values().find(|app| app.app_info.name.to_lowercase() == name_lower) {
            return Ok(Some(app.clone()));
        }
        
        // Finally try partial match
        if let Some(app) = registry.values().find(|app| {
            app.app_info.name.to_lowercase().contains(&name_lower) ||
            app.app_info.id.0.to_lowercase().contains(&name_lower)
        }) {
            return Ok(Some(app.clone()));
        }
        
        Ok(None)
    }
}

/// Parse a Veilid URI
pub fn parse_veil_uri(uri_str: &str) -> Result<VeilUri> {
    let url = url::Url::parse(uri_str)?;
    
    if url.scheme() != "veil" {
        return Err(color_eyre::eyre::eyre!("Invalid scheme: expected 'veil', got '{}'", url.scheme()));
    }
    
    // Handle both veil://app/... and veil:///app/... formats
    let path_segments = if let Some(host) = url.host_str() {
        // Format: veil://app/id/version (host=app, path=/id/version)
        let mut segments = vec![host];
        let path_parts: Vec<&str> = url.path().trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();
        segments.extend(path_parts);
        segments
    } else {
        // Format: veil:///app/id/version (no host, path=/app/id/version)
        url.path().trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect()
    };
    
    if path_segments.len() < 2 {
        return Err(color_eyre::eyre::eyre!("Invalid URI format: expected veil://app/<app_id>[/<version>] - not enough segments"));
    }
    
    if path_segments[0] != "app" {
        return Err(color_eyre::eyre::eyre!("Invalid URI format: expected veil://app/<app_id>[/<version>] - first segment should be 'app'"));
    }
    
    let app_id = AppId(path_segments[1].to_string());
    let version = if path_segments.len() >= 3 {
        Some(path_segments[2].to_string())
    } else {
        None
    };
    
    Ok(VeilUri::new(app_id, version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_veil_uri() {
        // Test with version
        let uri = parse_veil_uri("veil://app/my-app/1.0.0").unwrap();
        assert_eq!(uri.app_id.0, "my-app");
        assert_eq!(uri.version, Some("1.0.0".to_string()));
        
        // Test without version
        let uri = parse_veil_uri("veil://app/my-app").unwrap();
        assert_eq!(uri.app_id.0, "my-app");
        assert_eq!(uri.version, None);
        
        // Test invalid scheme
        assert!(parse_veil_uri("http://app/my-app").is_err());
        
        // Test invalid format
        assert!(parse_veil_uri("veil://invalid/format").is_err());
    }
} 