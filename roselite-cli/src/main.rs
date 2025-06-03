use clap::{Parser, Subcommand};
use color_eyre::Result;
use roselite_core::{
    package::{Package, PackageBuilder},
    store::{VeilidStore, AppStore},
    types::{VeilUri, AppId},
};
use std::path::PathBuf;
use url;
use std::fs;
use dirs;
use dialoguer::Password;
use std::env;
use std::io::{self, BufRead, Write};
use std::time::Duration;
use std::collections::HashMap;

mod gateway;

use gateway::UniversalGateway;

/// Roselite - P2P static site hosting via Veilid DHT
#[derive(Parser)]
#[command(name = "roselite")]
#[command(about = "Deploy static sites to the Veilid P2P network")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Base domain (host[:port]) of the universal gateway used to build shareable URLs
    /// (e.g. "example.com" or "localhost:8080"). Required for publish and access commands,
    /// but optional for bundle.
    #[arg(long = "gateway-url", global = true)]
    gateway_url: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Bundle a static site into a .veilidpkg package
    Bundle {
        /// Source directory containing the static site
        #[arg(value_name = "DIR")]
        source_dir: Option<PathBuf>,
        
        /// Output package file path
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        
        /// Site name
        #[arg(long)]
        name: Option<String>,
        
        /// Site version
        #[arg(long)]
        version: Option<String>,
        
        /// Site description
        #[arg(long)]
        description: Option<String>,
        
        /// Developer/author name
        #[arg(long)]
        developer: Option<String>,
        
        /// Entry point file (default: index.html)
        #[arg(long)]
        entry: Option<String>,
        
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    
    /// Publish a package to the Veilid DHT for P2P hosting
    Publish {
        /// Package file to publish
        #[arg(value_name = "PACKAGE")]
        package: PathBuf,
        
        /// Show all available gateways in output
        #[arg(short, long)]
        gateways: bool,
        
        /// Open the primary URL in browser after publishing
        #[arg(long)]
        open: bool,
    },
    
    /// Access a site directly from a DHT key or gateway URL
    Access {
        /// DHT key or gateway URL of the site to access
        #[arg(value_name = "KEY_OR_URL")]
        key_or_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("roselite=info,warn")
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Bundle { 
            source_dir, 
            output, 
            name, 
            version, 
            description, 
            developer, 
            entry, 
            tags 
        } => {
            cmd_bundle(
                source_dir, 
                output, 
                name, 
                version, 
                description, 
                developer, 
                entry, 
                tags
            ).await?;
        }
        Commands::Publish { package, gateways, open } => {
            ensure_password()?;
            let gw = cli.gateway_url.clone().ok_or_else(|| {
                color_eyre::eyre::eyre!("--gateway-url must be provided for publish")
            })?;
            cmd_publish(package, gateways, open, gw).await?;
        }
        Commands::Access { key_or_url } => {
            ensure_password()?;
            let gw = cli.gateway_url.clone().ok_or_else(|| {
                color_eyre::eyre::eyre!("--gateway-url must be provided for access")
            })?;
            cmd_access(key_or_url, gw).await?;
        }
    }

    Ok(())
}

async fn cmd_bundle(
    source_dir: Option<PathBuf>,
    output: Option<PathBuf>,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    developer: Option<String>,
    entry: Option<String>,
    tags: Option<String>,
) -> Result<()> {
    let source_dir = source_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    println!("🏗️  Bundling site from: {}", source_dir.display());
    
    // Interactive prompts for missing information
    let name = match name {
        Some(n) => n,
        None => {
            use dialoguer::Input;
            Input::new()
                .with_prompt("Site name")
                .interact_text()?
        }
    };
    
    let version = version.unwrap_or_else(|| "1.0.0".to_string());
    let entry = entry.unwrap_or_else(|| "index.html".to_string());
    
    // Build package
    let mut builder = PackageBuilder::new(name.clone(), &source_dir)
        .version(version.clone());
    
    if let Some(desc) = description {
        builder = builder.description(desc);
    }
    
    if let Some(dev) = developer {
        builder = builder.developer(dev);
    }
    
    builder = builder.entry(entry);
    
    if let Some(tags_str) = tags {
        let tag_list: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        builder = builder.tags(tag_list);
    }
    
    let package = builder.build().await?;
    
    // Output path
    let output_path = output.unwrap_or_else(|| {
        PathBuf::from(format!("{}.veilidpkg", name.replace(' ', "-").to_lowercase()))
    });
    
    // Save package to file
    tokio::fs::write(&output_path, &package.content).await?;
    
    println!("✅ Package created: {}", output_path.display());
    println!("📦 Size: {} bytes", std::fs::metadata(&output_path)?.len());
    
    Ok(())
}

async fn cmd_publish(package_path: PathBuf, gateways: bool, open: bool, gateway_domain: String) -> Result<()> {
    println!("📤 Publishing package: {}", package_path.display());
    
    // Load package
    let package = Package::from_file(&package_path).await?;
    
    println!("📦 Package: {} v{} by {}", 
        package.manifest.name, 
        package.manifest.version, 
        package.manifest.developer
    );
    
    // Initialize Veilid connection and store
    println!("🌐 Connecting to Veilid DHT...");
    let mut store = VeilidStore::new().await.map_err(|e| {
        println!("❌ Failed to connect to Veilid network: {}", e);
        color_eyre::eyre::eyre!("Unable to establish Veilid connection")
    })?;
    println!("✅ Successfully connected to Veilid network!");
    
    // Publish to Veilid DHT
    println!("📡 Publishing to Veilid DHT...");
    
    let result = match store.publish(package).await {
        Ok((veil_uri, updated_package)) => {
            println!("✅ Package published successfully!");
            
            // Get DHT key
            let dht_key = veil_uri.app_id.0.clone();
            println!("🔗 DHT key: {}", dht_key);
            println!("💡 Configure your domain with a DNS TXT record:\n    example.com. IN TXT \"veilid-app={}\"", dht_key);
            println!("👉 Then point a CNAME to your gateway ({}).", gateway_domain);
            
            // Show DHT record information
            println!("\n📊 DHT Record Details:");
            println!("   📋 App ID: {}", veil_uri.app_id.0);
            if let Some(version) = &veil_uri.version {
                println!("   📈 Version: {}", version);
            }
            println!("   🔗 DHT Record Key: {}", veil_uri.app_id.0);
            println!("   📡 Storage: Veilid distributed hash table");
            
            // Show instant web access
            println!("\n🚀 INSTANT WEB ACCESS:");
            println!("   🌐 Primary URL: {}", veil_uri.to_string());
            println!("   📱 Mobile-friendly access");
            println!("   🔄 Real-time DHT resolution");
            println!("   ✅ No setup required!");
            
            if gateways {
                // Show all available gateways
                println!("\n🌍 ALL AVAILABLE GATEWAYS:");
                println!("   🔗 Subdomain: {}", veil_uri.to_string());
                println!("   🔗 Direct DHT: http://{}/VLD0:{}", gateway_domain, veil_uri.app_id.0);
            }
            
            // Open in browser if requested
            if open {
                println!("\n🌐 Opening site in browser...");
                match open_url(&veil_uri.to_string()) {
                    Ok(_) => println!("✅ Opened {} in default browser", veil_uri.to_string()),
                    Err(e) => {
                        println!("⚠️  Failed to open browser: {}", e);
                        println!("💡 Manually visit: {}", veil_uri.to_string());
                    }
                }
            }
            
            // Traditional DNS setup (for advanced users)
            println!("\n🔧 ADVANCED: Custom Domain Setup (Optional)");
            println!("For your own domain (like {}):", veil_uri.to_string());
            println!("   1. Add DNS TXT record:");
            println!("      {}.com. IN TXT \"veilid-app={}\"", veil_uri.to_string(), veil_uri.app_id.0);
            if let Some(version) = &veil_uri.version {
                println!("      {}.com. IN TXT \"veilid-version={}\"", veil_uri.to_string(), version);
            }
            println!("   2. Point domain to gateway:");
            println!("      {}.com. CNAME {}", veil_uri.to_string(), gateway_domain);
            println!("   3. Access via: https://{}.com", veil_uri.to_string());
            
            println!("\n💡 Next Steps:");
            println!("   ✅ Your site is live at: {}", veil_uri.to_string());
            println!("   📤 Share the URL with users");
            if !open {
                println!("   🌐 Use --open flag to auto-launch browser");
            }
            if !gateways {
                println!("   🔗 Use --gateways flag to see all access options");
            }
            println!("   🚧 Note: The URL will resolve when your gateway at {} is accessible", gateway_domain);
            
            Ok(())
        },
        Err(e) => {
            println!("❌ Failed to publish package: {}", e);
            println!("💡 Try again later or check your network connection");
            Err(e.into())
        }
    };
    
    // Properly shutdown the store before returning
    println!("\n🔄 Disconnecting from Veilid network...");
    if let Err(e) = store.shutdown().await {
        println!("⚠️  Warning: Failed to shutdown cleanly: {}", e);
    } else {
        println!("✅ Disconnected successfully");
    }
    
    result
}

/// Open a URL in the default browser
fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", url])
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .output()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to open URL: {}", e))?;
    }
    
    Ok(())
}

async fn cmd_access(key_or_url: String, gateway_domain: String) -> Result<()> {
    println!("🌐 Accessing site: {}", key_or_url);
    
    let app_id = if key_or_url.starts_with("https://") || key_or_url.starts_with("http://") {
        // Extract domain and look up TXT record
        println!("🔍 Looking up DNS TXT record for domain...");
        println!("💡 In a complete implementation, this would:");
        println!("   • Extract veilid-app= value");
        println!("   • Use that as the DHT lookup key");
        
        // For now, extract from URL path or use domain as app ID
        let url = url::Url::parse(&key_or_url).map_err(|e| color_eyre::eyre::eyre!("Invalid URL: {}", e))?;
        let domain = url.host_str().unwrap_or("unknown");
        println!("📋 Domain: {}", domain);
        
        // Mock DHT key extraction (in reality would come from DNS TXT)
        AppId(domain.replace('.', "-"))
    } else {
        // Assume it's a direct DHT key
        AppId(key_or_url.clone())
    };
    
    println!("🔍 DHT Lookup Key: {}", app_id.0);
    
    // Initialize Veilid store to fetch site data
    println!("📡 Connecting to Veilid DHT...");
    let mut store = VeilidStore::new().await.map_err(|e| {
        println!("❌ Failed to connect to Veilid network: {}", e);
        color_eyre::eyre::eyre!("Unable to establish Veilid connection")
    })?;
    
    let result = async {
        // Try to fetch site from Veilid DHT
        match store.get_app(&app_id).await? {
            Some(app_info) => {
                println!("✅ Found site in Veilid DHT!");
                println!("📦 {}", app_info.name);
                println!("👨‍💻 Developer: {}", app_info.developer);
                println!("📈 Version: {}", app_info.version);
                println!("📝 Description: {}", app_info.description);
                
                // Show DNS integration info
                println!("\n🌐 DNS Integration:");
                println!("   📋 DHT Key: {}", app_id.0);
                println!("   🔗 Could be accessed via domain with TXT record:");
                println!("   example.com. IN TXT \"veilid-app={}\"", app_id.0);
                
                // Show gateway access information (but don't open browser)
                let gateway = UniversalGateway::from_domain(&gateway_domain);
                if let Ok(primary_url) = gateway.generate_url(&app_id, Some(&app_info.name)) {
                    println!("   🌐 Gateway URL: {}", primary_url);
                    
                    println!("\n📋 Access Information:");
                    println!("   🔗 Direct URL: {}", primary_url);
                    println!("   💡 You can visit this URL in any browser");
                    println!("   🌍 Content served via Veilid DHT");
                }
                
                // Try to download package and show technical details
                let uri = VeilUri::new(app_id.clone(), Some(app_info.version.clone()));
                match store.download(&uri).await {
                    Ok(package) => {
                        println!("\n📥 Successfully downloaded package from DHT");
                        println!("🚀 Site data retrieved via decentralized network");
                        
                        // Show technical details
                        println!("\n📊 DHT Access Details:");
                        println!("   📡 Retrieved from: Veilid distributed hash table");
                        println!("   🔑 DHT Key: {}", app_id.0);
                        println!("   📦 Package size: {} bytes", package.content.len());
                        println!("   🎯 Entry point: {}", package.manifest.entry);
                        
                        // For web sites, show how they could be served locally
                        if package.manifest.entry.contains(".html") || package.manifest.category.to_lowercase().contains("web") {
                            println!("\n🌐 Web Site Information:");
                            println!("   📄 Entry point: {}", package.manifest.entry);
                            println!("   🏷️  Category: {}", package.manifest.category);
                            println!("   💡 In a complete implementation, this would:");
                            println!("   • Extract the package to a temporary directory");
                            println!("   • Serve the site locally (e.g., http://localhost:8080)");
                            println!("   • All content served from DHT data (fully decentralized)");
                            println!("   • Or proxy through a Veilid gateway for direct domain access");
                        } else {
                            println!("\n💾 Static Site Information:");
                            println!("   💡 Would extract and serve appropriately based on content type");
                        }
                        
                        println!("\n🔗 Connection Summary:");
                        println!("   ✅ Site is accessible via DHT");
                        println!("   🌐 Gateway URL: {}", gateway.generate_url(&app_id, Some(&app_info.name)).unwrap_or_else(|_| "unavailable".to_string()));
                        println!("   📡 Served from: Veilid distributed network");
                        println!("   🔄 Status: Online and available");
                    },
                    Err(e) => {
                        println!("⚠️  Failed to download package: {}", e);
                        println!("📊 Site metadata is available, but package download failed");
                        
                        println!("\n🔗 Connection Summary:");
                        println!("   ⚠️  Partial access: metadata only");
                        println!("   🌐 Gateway URL: {}", gateway.generate_url(&app_id, Some(&app_info.name)).unwrap_or_else(|_| "unavailable".to_string()));
                        println!("   📡 Issue: Cannot retrieve full site data");
                    }
                }
            },
            None => {
                println!("📭 Site not found in Veilid DHT");
                println!("💡 This could mean:");
                println!("   • Site has not been published yet");
                println!("   • DHT key is incorrect");
                println!("   • DNS TXT record points to wrong key");
                println!("   • DHT propagation is still in progress");
                println!("   • Your Veilid node is not fully synchronized");
                
                println!("\n🔗 Connection Summary:");
                println!("   ❌ Site not accessible");
                println!("   📋 DHT Key: {}", app_id.0);
                println!("   📡 Status: Not found in network");
            }
        }
        
        Ok::<(), color_eyre::eyre::Error>(())
    }.await;
    
    // Properly shutdown the store before returning
    println!("\n🔄 Disconnecting from Veilid network...");
    if let Err(e) = store.shutdown().await {
        println!("⚠️  Warning: Failed to shutdown cleanly: {}", e);
    } else {
        println!("✅ Disconnected successfully");
    }
    
    result
}

fn ensure_password() -> Result<()> {
    if env::var("ROSELITE_PASSWORD").is_ok() {
        return Ok(());
    }

    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".roselite-config"));
    let file_path = config_dir.join("roselite").join("password.txt");
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let password = if file_path.exists() {
        fs::read_to_string(&file_path)?.trim().to_string()
    } else {
        let pass = Password::new()
            .with_prompt("Set a Veilid keystore password (leave blank to store unencrypted)")
            .with_confirmation("Confirm", "Passwords do not match")
            .allow_empty_password(true)
            .interact()?;
        fs::write(&file_path, &pass)?;
        pass
    };

    env::set_var("ROSELITE_PASSWORD", password);
    Ok(())
} 