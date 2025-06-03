use color_eyre::Result;
use roselite_core::types::AppId;
// use std::collections::HashMap;

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
            use_https: false,
            subdomain_prefix: None,
        }
    }
}

/// Universal Gateway manager for converting DHT keys to web URLs
pub struct UniversalGateway {
    config: GatewayConfig,
    // Previously we supported multiple "known" gateways for convenience.
    // The new design relies on a single, user-supplied gateway URL so this map is no longer needed.
    // Removing it simplifies the API and eliminates implicit behaviour.
}

impl UniversalGateway {
    /// Create a new Universal Gateway manager
    pub fn new() -> Self {
        Self {
            config: GatewayConfig::default(),
        }
    }

    /// Create gateway with user provided base domain (host[:port]). Use HTTPS if standard 443/8443 or if scheme "https://" is given.
    pub fn from_domain(domain_str: &str) -> Self {
        // Try to parse scheme
        let (clean_domain, use_https) = if let Some(stripped) = domain_str.strip_prefix("https://") {
            (stripped.to_string(), true)
        } else if let Some(stripped) = domain_str.strip_prefix("http://") {
            (stripped.to_string(), false)
        } else {
            // Heuristic: if port 8443 or no port implies https? else http.
            let https_guess = domain_str.ends_with(":443") || domain_str.ends_with(":8443");
            (domain_str.to_string(), https_guess)
        };

        let mut gw = Self::new();
        gw.config.domain = clean_domain;
        gw.config.use_https = use_https;
        gw
    }

    /// Generate a gateway URL for an app
    pub fn generate_url(&self, app_id: &AppId, app_name: Option<&str>) -> Result<String> {
        let subdomain = self.generate_subdomain(app_id, app_name);
        let protocol = if self.config.use_https { "https" } else { "http" };
        
        Ok(format!("{}://{}.{}", protocol, subdomain, self.config.domain))
    }

    /// Generate multiple gateway URLs for redundancy
    pub fn generate_all_urls(&self, app_id: &AppId, app_name: Option<&str>) -> Vec<(String, String)> {
        // With a single gateway configuration this returns only one entry — the primary URL.
        let url = self.generate_url(app_id, app_name).unwrap_or_default();
        vec![(self.config.domain.clone(), url)]
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
        if urls.len() <= 1 {
            "   (none)".to_string()
        } else {
            urls.iter()
                .map(|(name, url)| format!("   🔗 {}: {}", name, url))
                .collect::<Vec<_>>()
                .join("\n")
        }
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