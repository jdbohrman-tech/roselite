use clap::{Parser, Subcommand};
use color_eyre::Result;
use roselite_core::{
    package::{Package, PackageBuilder},
    store::{VeilidStore, AppStore},
    types::{AppInfo, VeilUri, AppId},
};
use std::path::PathBuf;
use tracing::{info, warn, debug};
use url;

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
        
        /// Show all available gateways
        #[arg(short, long)]
        gateways: bool,
        
        /// Open the site in browser after publishing
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
            cmd_publish(package, gateways, open).await?;
        }
        Commands::Access { key_or_url } => {
            cmd_access(key_or_url).await?;
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

async fn cmd_publish(package_path: PathBuf, gateways: bool, open: bool) -> Result<()> {
    println!("📤 Publishing package: {}", package_path.display());
    
    // Load package
    let package = Package::from_file(&package_path).await?;
    
    println!("📦 Package: {} v{} by {}", 
        package.manifest.name, 
        package.manifest.version, 
        package.manifest.developer
    );
    
    // Initialize Universal Gateway
    let gateway = UniversalGateway::new();
    
    // Initialize Veilid connection and store
    println!("🌐 Connecting to Veilid DHT...");
    let mut store = match VeilidStore::new().await {
        Ok(store) => {
            println!("✅ Successfully connected to Veilid network!");
            store
        },
        Err(e) => {
            println!("⚠️  Failed to connect to Veilid network: {}", e);
            println!("📝 This could be due to:");
            println!("   • No Veilid bootstrap nodes available");
            println!("   • Network connectivity issues");
            println!("   • Veilid node not properly configured");
            println!("   • Running in fallback mode");
            
            // Still try to proceed with fallback mode
            VeilidStore::new().await?
        }
    };
    
    // Publish to Veilid DHT
    println!("📡 Publishing to Veilid DHT...");
    
    match store.publish(package).await {
        Ok(veil_uri) => {
            println!("✅ Package published successfully!");
            
            // Show DHT record information
            println!("\n📊 DHT Record Details:");
            println!("   📋 App ID: {}", veil_uri.app_id.0);
            if let Some(version) = &veil_uri.version {
                println!("   📈 Version: {}", version);
            }
            println!("   🔗 DHT Record Key: {}", veil_uri.app_id.0);
            println!("   📡 Storage: Veilid distributed hash table");
            
            // Generate gateway URLs and instructions
            let app_name = Some(store.get_app(&veil_uri.app_id).await?.map(|app| app.name).unwrap_or_else(|| veil_uri.app_id.0.clone()));
            let primary_url = gateway.generate_url(&veil_uri.app_id, app_name.as_deref())?;
            
            // Show instant web access
            println!("\n🚀 INSTANT WEB ACCESS:");
            println!("   🌐 Primary URL: {}", primary_url);
            println!("   📱 Mobile-friendly HTTPS");
            println!("   🔄 Real-time DHT resolution");
            println!("   ✅ No setup required!");
            
            if gateways {
                // Show all available gateways
                println!("\n🌍 ALL AVAILABLE GATEWAYS:");
                let all_urls = gateway.generate_all_urls(&veil_uri.app_id, app_name.as_deref());
                for (name, url) in all_urls {
                    println!("   🔗 {}: {}", name, url);
                }
            }
            
            // Show comprehensive gateway instructions
            println!("\n{}", gateway.generate_setup_instructions(&veil_uri.app_id, app_name.as_deref()));
            
            // Show sharing information
            println!("\n{}", gateway.generate_sharing_text(&veil_uri.app_id, app_name.as_deref()));
            
            // Open in browser if requested
            if open {
                println!("\n🌐 Opening site in browser...");
                match open_url(&primary_url) {
                    Ok(_) => println!("✅ Opened {} in default browser", primary_url),
                    Err(e) => {
                        println!("⚠️  Failed to open browser: {}", e);
                        println!("💡 Manually visit: {}", primary_url);
                    }
                }
            }
            
            // Traditional DNS setup (for advanced users)
            println!("\n🔧 ADVANCED: Custom Domain Setup (Optional)");
            println!("For your own domain (like jdbohrman.tech):");
            println!("   1. Add DNS TXT record:");
            println!("      jdbohrman.tech. IN TXT \"veilid-app={}\"", veil_uri.app_id.0);
            if let Some(version) = &veil_uri.version {
                println!("      jdbohrman.tech. IN TXT \"veilid-version={}\"", version);
            }
            println!("   2. Deploy gateway code or use CNAME:");
            if let Ok(primary_url) = gateway.generate_url(&veil_uri.app_id, app_name.as_deref()) {
                let gateway_domain = primary_url.split("://").nth(1).unwrap_or("");
                println!("      jdbohrman.tech. CNAME {}", gateway_domain);
            }
            println!("   3. Access via: https://jdbohrman.tech");
            
            println!("\n💡 Next Steps:");
            println!("   ✅ Your site is live at: {}", primary_url);
            println!("   📤 Share the URL with users");
            if !open {
                println!("   🌐 Use --open flag to auto-launch browser");
            }
            if !gateways {
                println!("   🔗 Use --gateways flag to see all access options");
            }
        },
        Err(e) => {
            println!("❌ Failed to publish package: {}", e);
            println!("💡 Try again later or check your network connection");
            return Err(e.into());
        }
    }
    
    Ok(())
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

async fn cmd_access(key_or_url: String) -> Result<()> {
    println!("🌐 Accessing site: {}", key_or_url);
    
    let app_id = if key_or_url.starts_with("https://") || key_or_url.starts_with("http://") {
        // Extract domain and look up TXT record
        println!("🔍 Looking up DNS TXT record for domain...");
        println!("💡 In a complete implementation, this would:");
        println!("   • Query TXT records for the domain");
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
    let store = match VeilidStore::new().await {
        Ok(store) => store,
        Err(e) => {
            println!("⚠️  Failed to connect to Veilid network: {}", e);
            println!("📝 DHT access requires a connected Veilid node");
            println!("💡 To resolve DHT records, you need:");
            println!("   • A running Veilid node");
            println!("   • Network connectivity to DHT bootstrap nodes");
            println!("   • Proper Veilid configuration");
            
            println!("🚀 Would attempt to access site from DHT key: {}", app_id.0);
            return Ok(());
        }
    };
    
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
            
            // Show gateway access
            let gateway = UniversalGateway::new();
            if let Ok(primary_url) = gateway.generate_url(&app_id, Some(&app_info.name)) {
                println!("   🌐 Gateway URL: {}", primary_url);
                
                // Try to open in browser
                println!("\n🌐 Opening site in browser...");
                match open_url(&primary_url) {
                    Ok(_) => println!("✅ Opened {} in default browser", primary_url),
                    Err(e) => {
                        println!("⚠️  Failed to open browser: {}", e);
                        println!("💡 Manually visit: {}", primary_url);
                    }
                }
            }
            
            // Try to download package
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
                        println!("🌐 This is a web site");
                        println!("💡 In a complete implementation, this would:");
                        println!("   • Extract the package to a temporary directory");
                        println!("   • Serve the site locally (e.g., http://localhost:8080)");
                        println!("   • Launch the user's browser to the local URL");
                        println!("   • All content served from DHT data (fully decentralized)");
                        println!("   • Or proxy through a Veilid gateway for direct domain access");
                    } else {
                        println!("💾 This is a static site");
                        println!("💡 Would extract and serve appropriately based on content type");
                    }
                },
                Err(e) => {
                    println!("⚠️  Failed to download package: {}", e);
                    println!("📊 Site metadata is available, but package download failed");
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
        }
    }
    
    Ok(())
} 