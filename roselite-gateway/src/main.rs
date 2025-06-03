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
    process::{Command, Stdio},
};
use clap::Parser;
use anyhow::Result;
// Add DNS resolver
use hickory_resolver::{TokioAsyncResolver, config::{ResolverConfig, ResolverOpts}};

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

    /// Automatically start rust-rpxy in front of the HTTP service (provides automatic HTTPS)
    #[arg(long)]
    proxy: bool,

    /// Path to the rust-rpxy binary (defaults to `rpxy` in $PATH)
    #[arg(long, default_value = "rpxy")]
    proxy_bin: String,

    /// Email address used for ACME (Let's Encrypt) when --proxy is supplied
    #[arg(long, default_value = "admin@example.com")]
    acme_email: String,
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

/// Resolve Veilid DHT key for a domain via DNS TXT record `veilid-app=<KEY>`
async fn lookup_dht_key(domain: &str) -> Option<String> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    if let Ok(response) = resolver.txt_lookup(domain).await {
        for txt in response.iter() {
            for data in txt.txt_data() {
                if let Ok(text) = std::str::from_utf8(data) {
                    if let Some(rest) = text.strip_prefix("veilid-app=") {
                        return Some(rest.to_string());
                    }
                }
            }
        }
    }
    None
}

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
    
    // Create app cache and domain mapping
    let cache: AppCache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    
    // If proxy flag is set, spin up rust-rpxy in front of us BEFORE binding the HTTP listener.
    // This will forward :80 and :443 to our internal port.
    if args.proxy {
        start_rpxy(&args)?;

        info!("🛡️  rust-rpxy proxy launched. It will terminate TLS and forward traffic to the internal gateway.");
        info!("➡️  Make sure your domain's A/AAAA records point at this server's public IP.");
        info!("✅ Certificates will be obtained automatically via ACME for {}", args.domain);
    }
    
    // Build our application with routes
    let app = Router::new()
        .route("/", get(handle_root_request))
        .route("/*path", get(handle_path_request))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(CompressionLayer::new())
        )
        .with_state((state, cache));

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

/// Handle root domain requests (show welcome page)
async fn handle_root_request(
    Host(hostname): Host,
    State((state, cache)): State<(AppState, AppCache)>,
) -> impl IntoResponse {
    handle_request_internal(hostname, String::new(), state, cache).await
}

/// Handle requests with paths
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
    
    // Extract domain from subdomain
    let domain = match extract_domain_from_hostname(&hostname, &state.domain) {
        Some(domain) => domain,
        None => {
            // If no domain, this is a root domain request, show the welcome page
            info!("🏠 Root domain request: {}", hostname);
            return handle_root_response(&state).await;
        }
    };
    
    info!("🎯 Serving domain: {} (path: /{})", domain, path);
    
    // Resolve domain to DHT key via DNS TXT
    let dht_key = match lookup_dht_key(&domain).await {
        Some(key) => {
            info!("✅ Resolved domain '{}' to DHT key '{}' via DNS TXT", domain, key);
            key
        },
        None => {
            warn!("❌ No veilid-app TXT record found for domain: {}", domain);
            return handle_domain_not_found(&domain).await;
        }
    };
    
    // Try to get app from cache first
    {
        let cache_read = cache.read().await;
        if let Some(cached_app) = cache_read.get(&domain) {
            debug!("💾 Found app in cache: {}", domain);
            return serve_static_file(&cached_app.extract_path, &path).await;
        }
    }
    
    // Check if app directory exists in cache (for manual testing or fallback)
    let extract_path = state.cache_dir.join(&domain);
    if extract_path.exists() && extract_path.is_dir() {
        info!("📁 Found app directory in cache: {}", domain);
        return serve_static_file(&extract_path, &path).await;
    }
    
    // Not in cache, fetch from DHT using the resolved DHT key
    info!("📡 Fetching app from Veilid DHT using key: {}", dht_key);
    
    let store = state.store.lock().await;
    
    // Create VeilUri from the DHT key
    let app_id = AppId(dht_key.clone());
    let uri = VeilUri::new(app_id, None);
    
    match store.download(&uri).await {
        Ok(package) => {
            info!("✅ Successfully downloaded package for domain: {}", domain);
            
            // Extract package to cache directory
            let extract_path = state.cache_dir.join(&domain);
            if let Err(e) = extract_package_to_cache(&package, &extract_path).await {
                error!("❌ Failed to extract package {}: {}", domain, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to extract app").into_response();
            }
            
            // Cache the app
            let cached_app = CachedApp {
                package: package.clone(),
                extract_path: extract_path.clone(),
                last_accessed: std::time::Instant::now(),
            };
            
            {
                let mut cache_write = cache.write().await;
                cache_write.insert(domain.clone(), cached_app);
            }
            
            info!("💾 Cached app: {}", domain);
            
            // Serve the requested file
            serve_static_file(&extract_path, &path).await
        }
        Err(e) => {
            error!("❌ Failed to fetch app {}: {}", domain, e);
            handle_domain_not_found(&domain).await
        }
    }
}

/// Handle unknown domain
async fn handle_domain_not_found(domain: &str) -> Response {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Site Not Found</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .container {{
            background: white;
            padding: 40px;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
            text-align: center;
            max-width: 500px;
        }}
        h1 {{ color: #e74c3c; margin-bottom: 20px; }}
        p {{ margin: 10px 0; line-height: 1.6; }}
        .code {{ 
            background: #f8f9fa; 
            padding: 10px; 
            border-radius: 5px; 
            font-family: monospace; 
            margin: 15px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔍 Site Not Found</h1>
        <p>The site <strong>{}</strong> is not published or does not have a valid DNS TXT record.</p>
        <p>If you're the owner, publish your site using:</p>
        <div class="code">roselite publish your-site.veilidpkg</div>
        <p>Then add a DNS TXT record pointing to your DHT key.</p>
    </div>
</body>
</html>"#,
        domain
    );
    
    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

/// Handle app download failure
async fn handle_app_not_found(domain: &str) -> Response {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>App Download Failed</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .container {{
            background: white;
            padding: 40px;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
            text-align: center;
            max-width: 500px;
        }}
        h1 {{ color: #e67e22; margin-bottom: 20px; }}
        p {{ margin: 10px 0; line-height: 1.6; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>⚠️ Download Failed</h1>
        <p>Failed to download the app <strong>{}</strong> from the Veilid DHT.</p>
        <p>This could be a temporary network issue or the content may no longer be available.</p>
        <p>Please try again later or contact the site owner.</p>
    </div>
</body>
</html>"#,
        domain
    );
    
    (StatusCode::SERVICE_UNAVAILABLE, Html(html)).into_response()
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
        <h2>🚀 Quick Start</h2>
        <p>To serve your Veilid app, you have a few options:</p>
        
        <h3>📦 Option 1: Publish and Access via Domain</h3>
        <ol>
            <li>Publish your app: <code>roselite publish your-app.veilidpkg</code></li>
            <li>Set up DNS TXT record: <code>your-domain.com. IN TXT "veilid-app=YOUR_DHT_KEY"</code></li>
            <li>Access via: <div class="example">http://your-domain.{}</div></li>
        </ol>
        
        <h3>🔧 Option 2: Direct DHT Access</h3>
        <p>Use any subdomain format:</p>
        <div class="example">http://your-app-domain.{}</div>
        <p>Where <code>your-app-domain</code> has a DNS TXT record pointing to your DHT key.</p>
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
    "#, state.domain, state.domain, state.domain, state.domain);
    
    Html(html).into_response()
}

/// Extract domain from hostname (e.g., "my-app.localhost:8080" -> "my-app")
fn extract_domain_from_hostname(hostname: &str, domain: &str) -> Option<String> {
    let hostname_no_port = hostname.split(':').next().unwrap_or(hostname);
    let domain_no_port = domain.split(':').next().unwrap_or(domain);
    
    if hostname_no_port == domain_no_port {
        None // Root domain
    } else {
        Some(hostname_no_port.to_string())
    }
}

/// Extract package contents to cache directory
async fn extract_package_to_cache(package: &Package, extract_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Remove existing directory if it exists
    if extract_path.exists() {
        std::fs::remove_dir_all(extract_path)?;
    }
    
    // Create the directory
    std::fs::create_dir_all(extract_path)?;
    
    // Extract the archive contents - use package.content which contains the gzipped tar
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    
    let decoder = GzDecoder::new(Cursor::new(&package.content));
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(extract_path)?;
    
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

/// Generate a temporary rpxy configuration and spawn the proxy process in the background.
/// Returns the child process handle (but the caller may choose to detach).
fn start_rpxy(args: &Args) -> Result<()> {
    // Build minimal TOML configuration for rpxy
    let config_toml = format!(
        r#"listen_port = 80
listen_port_tls = 443

[apps."gateway"]
server_name = "{domain}"
reverse_proxy = [{{ upstream = [{{ location = "127.0.0.1:{backend_port}", tls = false }}] }}]
tls = {{ https_redirection = true, acme = true }}

[experimental.acme]
email = "{email}"
+"#,
        domain = args.domain,
        backend_port = args.port,
        email = args.acme_email,
    );

    // Persist config to disk next to the running process (or in cache dir).
    let config_path = std::env::current_dir()?.join("rpxy-gateway-config.toml");
    std::fs::write(&config_path, config_toml.as_bytes())?;

    // Spawn the rpxy process; inherit stdout/stderr for visibility.
    let mut child = Command::new(&args.proxy_bin)
        .arg("--config")
        .arg(config_path.to_str().expect("valid temp path"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to launch rpxy: {}", e))?;

    // Detach: we don't await, but we also don't want child to be dropped immediately.
    // Spawn a background task that simply waits and logs on exit.
    std::thread::spawn(move || {
        if let Ok(status) = child.wait() {
            if status.success() {
                info!("rust-rpxy exited cleanly");
            } else {
                error!("rust-rpxy process exited with status {:?}", status);
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_from_hostname() {
        let domain = "localhost:8080";
        
        // Test valid subdomains
        assert_eq!(extract_domain_from_hostname("my-app.localhost:8080", domain), Some("my-app".to_string()));
        assert_eq!(extract_domain_from_hostname("test-site.localhost:8080", domain), Some("test-site".to_string()));
        assert_eq!(extract_domain_from_hostname("localhost:8080", domain), None);
        assert_eq!(extract_domain_from_hostname("invalid.com", domain), None);
        assert_eq!(extract_domain_from_hostname("sub.my-app.localhost:8080", domain), Some("sub.my-app".to_string()));
        
        // Test without port
        assert_eq!(extract_domain_from_hostname("my-app.localhost", "localhost"), Some("my-app".to_string()));
        assert_eq!(extract_domain_from_hostname("localhost", "localhost"), None);
        
        // Test with production domain
        let prod_domain = "roselite.app";
        assert_eq!(extract_domain_from_hostname("my-app.roselite.app", prod_domain), Some("my-app".to_string()));
        assert_eq!(extract_domain_from_hostname("roselite.app", prod_domain), None);
    }
} 