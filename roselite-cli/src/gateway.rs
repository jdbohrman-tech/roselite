use color_eyre::Result;
use roselite_core::types::AppId;
use std::collections::HashMap;

/// Universal Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub domain: String,
    pub use_https: bool,
    pub subdomain_prefix: Option<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            domain: "localhost:8080".to_string(),
            use_https: true,
            subdomain_prefix: None,
        }
    }
}

/// Universal Gateway manager for converting DHT keys to web URLs
pub struct UniversalGateway {
    config: GatewayConfig,
    known_gateways: HashMap<String, GatewayConfig>,
}

impl UniversalGateway {
    /// Create a new Universal Gateway manager
    pub fn new() -> Self {
        let mut known_gateways = HashMap::new();
        
        // Add known public gateways
        known_gateways.insert(
            "localhost:8080".to_string(),
            GatewayConfig {
                domain: "localhost:8080".to_string(),
                use_https: true,
                subdomain_prefix: None,
            }
        );
        
        known_gateways.insert(
            "roselite.app".to_string(),
            GatewayConfig {
                domain: "roselite.app".to_string(),
                use_https: true,
                subdomain_prefix: None,
            }
        );
        
        // Add localhost for development
        known_gateways.insert(
            "localhost".to_string(),
            GatewayConfig {
                domain: "localhost:3000".to_string(),
                use_https: false,
                subdomain_prefix: Some("dht".to_string()),
            }
        );

        Self {
            config: GatewayConfig::default(),
            known_gateways,
        }
    }

    /// Generate a gateway URL for an app
    pub fn generate_url(&self, app_id: &AppId, app_name: Option<&str>) -> Result<String> {
        let subdomain = self.generate_subdomain(app_id, app_name);
        let protocol = if self.config.use_https { "https" } else { "http" };
        
        Ok(format!("{}://{}.{}", protocol, subdomain, self.config.domain))
    }

    /// Generate a gateway URL using slug if available
    pub fn generate_url_with_slug(&self, app_id: &AppId, slug: Option<&str>, app_name: Option<&str>) -> Result<String> {
        let subdomain = if let Some(slug_val) = slug.filter(|s| !s.is_empty()) {
            slug_val.to_string()
        } else {
            self.generate_subdomain(app_id, app_name)
        };
        let protocol = if self.config.use_https { "https" } else { "http" };
        
        Ok(format!("{}://{}.{}", protocol, subdomain, self.config.domain))
    }

    /// Generate multiple gateway URLs for redundancy
    pub fn generate_all_urls(&self, app_id: &AppId, app_name: Option<&str>) -> Vec<(String, String)> {
        let mut urls = Vec::new();
        
        for (name, config) in &self.known_gateways {
            let subdomain = self.generate_subdomain_for_config(app_id, app_name, config);
            let protocol = if config.use_https { "https" } else { "http" };
            let url = format!("{}://{}.{}", protocol, subdomain, config.domain);
            urls.push((name.clone(), url));
        }
        
        urls
    }

    /// Generate gateway setup instructions
    pub fn generate_setup_instructions(&self, app_id: &AppId, app_name: Option<&str>) -> String {
        let subdomain = self.generate_subdomain(app_id, app_name);
        let primary_url = self.generate_url(app_id, app_name).unwrap_or_default();
        
        format!(
r#"🌐 Universal Gateway Access:

✅ INSTANT ACCESS (No setup required):
   🔗 Primary: {}
   📱 Mobile friendly with HTTPS
   🚀 Automatic DHT resolution
   
🌍 Alternative Gateways:
{}

💡 How it works:
   • Gateway resolves {} to DHT key: {}
   • Fetches content from Veilid DHT in real-time
   • Serves over HTTPS with proper caching
   • No DNS setup required on your part

🔧 For your own domain (optional):
   • Add DNS TXT: your-domain.com. IN TXT "veilid-app={}"
   • Deploy gateway code (see docs)
   • Or use DNS CNAME: your-domain.com. CNAME {}.{}"#,
            primary_url,
            self.format_alternative_gateways(app_id, app_name),
            subdomain,
            app_id.0,
            app_id.0,
            subdomain,
            self.config.domain
        )
    }

    /// Generate subdomain from app ID and name
    fn generate_subdomain(&self, app_id: &AppId, app_name: Option<&str>) -> String {
        self.generate_subdomain_for_config(app_id, app_name, &self.config)
    }

    /// Generate subdomain for a specific gateway config
    fn generate_subdomain_for_config(&self, app_id: &AppId, app_name: Option<&str>, config: &GatewayConfig) -> String {
        // Use app name if available, otherwise use app ID
        let base = if let Some(name) = app_name {
            // Convert app name to URL-safe subdomain
            name.to_lowercase()
                .replace(' ', "-")
                .replace('_', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>()
                .trim_matches('-')
                .to_string()
        } else {
            // Use first 12 chars of app ID for shorter subdomains
            app_id.0.chars().take(12).collect()
        };

        // Add prefix if configured
        if let Some(prefix) = &config.subdomain_prefix {
            format!("{}-{}", prefix, base)
        } else {
            base
        }
    }

    /// Format alternative gateways list
    fn format_alternative_gateways(&self, app_id: &AppId, app_name: Option<&str>) -> String {
        let urls = self.generate_all_urls(app_id, app_name);
        urls.iter()
            .map(|(name, url)| format!("   🔗 {}: {}", name, url))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate sharing text with multiple access methods
    pub fn generate_sharing_text(&self, app_id: &AppId, app_name: Option<&str>) -> String {
        let primary_url = self.generate_url(app_id, app_name).unwrap_or_default();
        
        format!(
r#"🚀 Share your app:

🌐 Web Access: {}
🔗 DHT Key: {}

💡 Users can access via:
• Web browser (any device)
• Veilid-native apps
• Direct DHT lookup"#,
            primary_url,
            app_id.0
        )
    }
}

impl Default for UniversalGateway {
    fn default() -> Self {
        Self::new()
    }
} 