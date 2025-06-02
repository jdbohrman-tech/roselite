use roselite_core::{
    store::{AppStore, VeilidStore},
    types::{AppId, VeilUri}, 
    package::Package,
};
use axum::{
    extract::{Host, Path, State},
    response::{Html, Response, IntoResponse},
    routing::get,
    Router,
    http::{StatusCode, HeaderMap},
};
use axum_server::tls_rustls::RustlsConfig;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn, error, debug};
use std::{
    sync::Arc,
    path::PathBuf,
    collections::HashMap,
};
use clap::Parser;
use anyhow::Result;

/// Veilid Gateway Server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// HTTP port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// HTTPS port to listen on
    #[arg(long, default_value = "8443")]
    https_port: u16,
    
    /// Domain to serve (for wildcard subdomain matching)
    #[arg(short, long, default_value = "localhost:8080")]
    domain: String,
    
    /// Path to TLS certificate file
    #[arg(long)]
    cert_file: Option<PathBuf>,
    
    /// Path to TLS private key file
    #[arg(long)]
    key_file: Option<PathBuf>,
    
    /// Enable HTTPS (requires cert and key files)
    #[arg(long)]
    enable_https: bool,
    
    /// Cache directory for extracted apps
    #[arg(long, default_value = ".cache")]
    cache_dir: String,
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    store: Arc<tokio::sync::Mutex<VeilidStore>>,
    cache_dir: PathBuf,
    domain: String,
}

/// Cached app information
#[derive(Clone, Debug)]
struct CachedApp {
    package: Package,
    extract_path: PathBuf,
    last_accessed: std::time::Instant,
}

type AppCache = Arc<tokio::sync::RwLock<HashMap<String, CachedApp>>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,roselite_gateway=debug,roselite_core=debug".into()),
        )
        .init();

    let args = Args::parse();
    
    info!("🚀 Starting Roselite Veilid Gateway");
    info!("📡 Connecting to Veilid DHT...");
    
    // Initialize Veilid store
    let store = Arc::new(tokio::sync::Mutex::new(VeilidStore::new().await?));
    
    // Create cache directory
    let cache_dir = PathBuf::from(&args.cache_dir);
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    
    // Create shared state
    let state = AppState {
        store,
        cache_dir,
        domain: args.domain.clone(),
    };
    
    // Create app cache
    let app_cache: AppCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    
    // Build our application with routes
    let app = Router::new()
        .route("/*path", get(handle_path_request))
        .route("/", get(handle_root_request))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(CompressionLayer::new())
        )
        .with_state((state, app_cache));

    // Start HTTP server
    let http_addr = format!("0.0.0.0:{}", args.port);
    info!("🌐 HTTP server listening on {}", http_addr);
    
    if args.enable_https {
        // Start HTTPS server if certificates are provided
        if let (Some(cert_file), Some(key_file)) = (args.cert_file, args.key_file) {
            let https_addr = format!("0.0.0.0:{}", args.https_port);
            info!("🔒 HTTPS server listening on {}", https_addr);
            
            let config = RustlsConfig::from_pem_file(cert_file, key_file).await?;
            
            // Start both HTTP and HTTPS servers concurrently
            tokio::try_join!(
                axum::serve(
                    tokio::net::TcpListener::bind(&http_addr).await?,
                    app.clone()
                ),
                axum_server::bind_rustls(https_addr.parse()?, config)
                    .serve(app.into_make_service())
            )?;
        } else {
            error!("HTTPS enabled but certificate or key file not provided");
            return Err(anyhow::anyhow!("Missing TLS certificate files"));
        }
    } else {
        // Start only HTTP server
        info!("⚠️  Running in HTTP-only mode (no TLS)");
        axum::serve(
            tokio::net::TcpListener::bind(http_addr).await?,
            app
        ).await?;
    }
    
    Ok(())
}

/// Handle root path requests
async fn handle_root_request(
    Host(hostname): Host,
    State((state, cache)): State<(AppState, AppCache)>,
) -> impl IntoResponse {
    handle_request_internal(hostname, String::new(), state, cache).await
}

/// Handle path requests
async fn handle_path_request(
    Host(hostname): Host,
    Path(path): Path<String>,
    State((state, cache)): State<(AppState, AppCache)>,
) -> impl IntoResponse {
    handle_request_internal(hostname, path, state, cache).await
}

/// Internal request handler
async fn handle_request_internal(
    hostname: String,
    path: String,
    state: AppState,
    cache: AppCache,
) -> impl IntoResponse {
    debug!("📡 Request: {} -> /{}", hostname, path);
    
    // Extract slug from subdomain
    let slug = match extract_slug_from_hostname(&hostname, &state.domain) {
        Some(slug) => slug,
        None => {
            // If no slug, this is a root domain request, show the welcome page
            info!("🏠 Root domain request: {}", hostname);
            return handle_root_response(&state).await;
        }
    };
    
    info!("🎯 Serving app: {} (path: /{})", slug, path);
    
    // Try to get app from cache first
    {
        let cache_read = cache.read().await;
        if let Some(cached_app) = cache_read.get(&slug) {
            debug!("💾 Found app in cache: {}", slug);
            return serve_static_file(&cached_app.extract_path, &path).await;
        }
    }
    
    // Check if app directory exists in cache (for manual testing or fallback)
    let extract_path = state.cache_dir.join(&slug);
    if extract_path.exists() && extract_path.is_dir() {
        info!("📁 Found app directory in cache: {}", slug);
        return serve_static_file(&extract_path, &path).await;
    }
    
    // Not in cache, fetch from DHT
    info!("📡 Fetching app from Veilid DHT: {}", slug);
    
    let package = {
        let store = state.store.lock().await;
        match store.download(&VeilUri::new(AppId(slug.clone()), None)).await {
            Ok(package) => package,
            Err(e) => {
                error!("❌ Failed to fetch app {}: {}", slug, e);
                return (StatusCode::NOT_FOUND, format!("App not found: {}", slug)).into_response();
            }
        }
    };
    
    // Extract package to cache directory
    if let Err(e) = extract_package_to_cache(&package, &extract_path).await {
        error!("❌ Failed to extract package {}: {}", slug, e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to extract app").into_response();
    }
    
    // Add to cache
    {
        let mut cache_write = cache.write().await;
        cache_write.insert(slug.clone(), CachedApp {
            package: package.clone(),
            extract_path: extract_path.clone(),
            last_accessed: std::time::Instant::now(),
        });
    }
    
    info!("✅ Cached app: {}", slug);
    
    // Serve the requested file
    serve_static_file(&extract_path, &path).await
}

/// Generate root response HTML
async fn handle_root_response(state: &AppState) -> Response {
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Roselite Veilid Gateway - Local Development</title>
    <style>
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px; 
            margin: 50px auto; 
            padding: 20px;
            background: #f8fafc;
            color: #1a202c;
        }}
        .header {{ 
            text-align: center; 
            margin-bottom: 40px;
            padding: 30px;
            background: white;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.05);
        }}
        .info {{ 
            background: white; 
            padding: 25px; 
            margin: 20px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
        }}
        .code {{ 
            background: #f1f5f9; 
            padding: 15px; 
            border-radius: 6px; 
            font-family: 'SF Mono', Consolas, monospace;
            border-left: 4px solid #3b82f6;
        }}
        .example {{ color: #3b82f6; font-weight: 600; }}
        .dev-note {{ 
            background: #fef3c7; 
            border: 1px solid #f59e0b; 
            border-radius: 6px; 
            padding: 12px; 
            margin: 15px 0;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 Roselite Veilid Gateway</h1>
        <p>Local Development Server</p>
        <p>Decentralized App Gateway powered by Veilid DHT</p>
    </div>
    
    <div class="dev-note">
        <strong>💻 Development Mode:</strong> This server is running locally for development and testing.
    </div>
    
    <div class="info">
        <h2>📡 Gateway Status</h2>
        <p>✅ Connected to Veilid DHT</p>
        <p>🌐 Serving on: <code>{}</code></p>
        <p>🚀 Ready to serve decentralized apps</p>
    </div>
    
    <div class="info">
        <h2>🎯 How to Access Apps Locally</h2>
        <p>Apps are served via subdomain routing. For local development:</p>
        <div class="code">
            <div class="example">http://your-app-slug.localhost:8080</div>
        </div>
        <p>Where <code>your-app-slug</code> is the slug of your published app.</p>
        
        <div class="dev-note">
            <strong>📝 Note:</strong> Modern browsers support <code>*.localhost</code> subdomains automatically. 
            No DNS configuration needed!
        </div>
    </div>
    
    <div class="info">
        <h2>🛠️ Development Workflow</h2>
        <p>1. Bundle your static site into a Veilid package:</p>
        <div class="code">
            roselite bundle ./my-website --name "My Website" --developer "Your Name"
        </div>
        
        <p>2. Publish to the local Veilid DHT:</p>
        <div class="code">
            roselite publish my-website.veilidpkg
        </div>
        
        <p>3. Access your app locally:</p>
        <div class="code">
            <div class="example">http://my-website.localhost:8080</div>
        </div>
    </div>
    
    <div class="info">
        <h2>📁 Manual Testing</h2>
        <p>For quick testing, you can also place app directories directly in the cache:</p>
        <div class="code">
            mkdir -p .cache/my-test-app<br>
            cp -r /path/to/your/static/site/* .cache/my-test-app/<br>
            # Visit: http://my-test-app.localhost:8080
        </div>
    </div>
    
    <div class="info">
        <h2>🔧 Server Configuration</h2>
        <p>Current settings:</p>
        <ul>
            <li><strong>Port:</strong> 8080</li>
            <li><strong>Domain:</strong> {}</li>
            <li><strong>Cache Directory:</strong> .cache/</li>
            <li><strong>HTTPS:</strong> Disabled (development mode)</li>
        </ul>
    </div>
</body>
</html>
    "#, state.domain, state.domain);
    
    Html(html).into_response()
}

/// Extract slug from hostname (e.g., "my-app.localhost:8080" -> "my-app")
fn extract_slug_from_hostname(hostname: &str, domain: &str) -> Option<String> {
    // Remove port from hostname if present
    let hostname_no_port = hostname.split(':').next().unwrap_or(hostname);
    let domain_no_port = domain.split(':').next().unwrap_or(domain);
    
    debug!("🔍 Extracting slug from hostname: '{}' (no port: '{}'), domain: '{}' (no port: '{}')", 
           hostname, hostname_no_port, domain, domain_no_port);
    
    if hostname_no_port == domain_no_port {
        debug!("🏠 Root domain detected");
        return None; // Root domain
    }
    
    if let Some(subdomain) = hostname_no_port.strip_suffix(&format!(".{}", domain_no_port)) {
        if subdomain.is_empty() {
            debug!("❌ Empty subdomain");
            None
        } else {
            debug!("✅ Extracted slug: '{}'", subdomain);
            Some(subdomain.to_string())
        }
    } else {
        debug!("❌ Hostname doesn't match domain pattern");
        None
    }
}

/// Extract package contents to cache directory
async fn extract_package_to_cache(package: &Package, extract_path: &PathBuf) -> Result<()> {
    // Create extraction directory
    if extract_path.exists() {
        std::fs::remove_dir_all(extract_path)?;
    }
    std::fs::create_dir_all(extract_path)?;
    
    // Extract files from package using the extract_files method
    let files = package.extract_files().await?;
    
    // Write each file to the extraction directory
    for (file_path, file_data) in &files {
        let full_path = extract_path.join(file_path);
        
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Write file
        std::fs::write(full_path, file_data)?;
    }
    
    debug!("📁 Extracted {} files to {:?}", files.len(), extract_path);
    Ok(())
}

/// Serve static file from extracted app
async fn serve_static_file(base_path: &PathBuf, requested_path: &str) -> Response {
    let mut file_path = base_path.clone();
    
    // Handle root path or empty path
    let clean_path = if requested_path.is_empty() || requested_path == "/" {
        "index.html"
    } else {
        requested_path.trim_start_matches('/')
    };
    
    file_path.push(clean_path);
    
    // Security check: ensure path is within base directory
    if !file_path.starts_with(base_path) {
        warn!("🚨 Path traversal attempt: {}", requested_path);
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }
    
    // Check if file exists
    if !file_path.exists() {
        debug!("❌ File not found: {:?}", file_path);
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }
    
    // Read file
    match std::fs::read(&file_path) {
        Ok(contents) => {
            // Determine content type
            let content_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();
            
            let mut headers = HeaderMap::new();
            headers.insert("content-type", content_type.parse().unwrap());
            
            debug!("✅ Serving file: {:?} ({} bytes)", file_path, contents.len());
            (StatusCode::OK, headers, contents).into_response()
        }
        Err(e) => {
            error!("❌ Failed to read file {:?}: {}", file_path, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_slug_from_hostname() {
        // Test with localhost development setup
        let domain = "localhost:8080";
        
        assert_eq!(extract_slug_from_hostname("my-app.localhost:8080", domain), Some("my-app".to_string()));
        assert_eq!(extract_slug_from_hostname("test-site.localhost:8080", domain), Some("test-site".to_string()));
        assert_eq!(extract_slug_from_hostname("localhost:8080", domain), None);
        assert_eq!(extract_slug_from_hostname("invalid.com", domain), None);
        assert_eq!(extract_slug_from_hostname("sub.my-app.localhost:8080", domain), Some("sub.my-app".to_string()));
        
        // Test without port for browser compatibility
        assert_eq!(extract_slug_from_hostname("my-app.localhost", "localhost"), Some("my-app".to_string()));
        assert_eq!(extract_slug_from_hostname("localhost", "localhost"), None);
        
        // Test production domain for reference
        let prod_domain = "localhost:8080";
        assert_eq!(extract_slug_from_hostname("my-app.localhost:8080", prod_domain), Some("my-app".to_string()));
        assert_eq!(extract_slug_from_hostname("localhost:8080", prod_domain), None);
    }
} 