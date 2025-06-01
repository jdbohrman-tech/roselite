use clap::{Parser, Subcommand};
use color_eyre::Result;
use roselite_core::*;
use std::path::PathBuf;
use std::io::Read;

mod local;

use local::{LocalRegistry, parse_veil_uri};
use roselite_core::store::AppStore;

/// Roselite - Decentralized app store for Veilid
#[derive(Parser)]
#[command(name = "roselite")]
#[command(about = "A decentralized app store for the Veilid network")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bundle an app into a .veilidpkg package
    Bundle {
        /// Source directory containing the app
        #[arg(value_name = "DIR")]
        source_dir: Option<PathBuf>,
        
        /// Output package file path
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        
        /// App name
        #[arg(long)]
        name: Option<String>,
        
        /// App version
        #[arg(long)]
        version: Option<String>,
        
        /// App description
        #[arg(long)]
        description: Option<String>,
        
        /// Developer name
        #[arg(long)]
        developer: Option<String>,
        
        /// Entry point file
        #[arg(long)]
        entry: Option<String>,
        
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    
    /// Publish a package to the Veilid DHT
    Publish {
        /// Package file to publish
        #[arg(value_name = "PACKAGE")]
        package: PathBuf,
    },
    
    /// Install an app from a veil:// URI
    Install {
        /// Veilid URI of the app to install
        #[arg(value_name = "URI")]
        uri: String,
    },
    
    /// List installed apps
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Run an installed app
    Run {
        /// Name or ID of the app to run
        #[arg(value_name = "APP")]
        app: String,
    },
    
    /// Search for apps in the store
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Filter by tags
        #[arg(long)]
        tags: Option<String>,
        
        /// Filter by developer
        #[arg(long)]
        developer: Option<String>,
        
        /// Maximum number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },
    
    /// Show app information
    Info {
        /// App ID or veil:// URI
        #[arg(value_name = "APP")]
        app: String,
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
        Commands::Publish { package } => {
            cmd_publish(package).await?;
        }
        Commands::Install { uri } => {
            cmd_install(uri).await?;
        }
        Commands::List { verbose } => {
            cmd_list(verbose).await?;
        }
        Commands::Run { app } => {
            cmd_run(app).await?;
        }
        Commands::Search { query, tags, developer, limit } => {
            cmd_search(query, tags, developer, limit).await?;
        }
        Commands::Info { app } => {
            cmd_info(app).await?;
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
    
    println!("🏗️  Bundling app from: {}", source_dir.display());
    
    // Interactive prompts for missing information
    let name = match name {
        Some(n) => n,
        None => {
            use dialoguer::Input;
            Input::new()
                .with_prompt("App name")
                .interact_text()?
        }
    };
    
    let version = version.unwrap_or_else(|| "1.0.0".to_string());
    let entry = entry.unwrap_or_else(|| "index.html".to_string());
    
    // Build package
    let mut builder = PackageBuilder::new(name.clone(), &source_dir)
        .version(version)
        .entry(entry);
        
    if let Some(desc) = description {
        builder = builder.description(desc);
    }
    
    if let Some(dev) = developer {
        builder = builder.developer(dev);
    }
    
    if let Some(tags_str) = tags {
        let tag_list: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        builder = builder.tags(tag_list);
    }
    
    let package = builder.build().await?;
    
    let output_path = output.unwrap_or_else(|| {
        PathBuf::from(format!("{}{}", name, PACKAGE_EXTENSION))
    });
    
    // Actually write the package to disk
    tokio::fs::write(&output_path, &package.content).await?;
    
    println!("✅ Package created: {}", output_path.display());
    println!("📦 Size: {} bytes", package.size_bytes);
    
    Ok(())
}

async fn cmd_publish(package_path: PathBuf) -> Result<()> {
    println!("📤 Publishing package: {}", package_path.display());
    
    let package = Package::from_file(&package_path).await?;
    let mut store = store::VeilidStore::new().await?;
    
    let uri = store.publish(package).await?;
    
    println!("✅ Published successfully!");
    println!("🔗 URI: {}", uri);
    
    // Generate QR code
    match generate_qr_code(&uri.to_string()) {
        Ok(qr_text) => {
            println!("📱 QR Code:");
            println!("{}", qr_text);
        }
        Err(e) => {
            println!("⚠️  Failed to generate QR code: {}", e);
        }
    }
    
    Ok(())
}

/// Generate a QR code for the given text
fn generate_qr_code(text: &str) -> Result<String> {
    use qrcode::{QrCode, render::unicode};
    
    let code = QrCode::new(text)?;
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    
    Ok(image)
}

async fn cmd_install(uri_str: String) -> Result<()> {
    println!("📥 Installing app from: {}", uri_str);
    
    // Parse the Veilid URI
    let uri = parse_veil_uri(&uri_str)?;
    println!("🔍 Parsed URI: App ID = {}, Version = {:?}", uri.app_id, uri.version);
    
    // Initialize store and local registry
    let store = store::VeilidStore::new().await?;
    let registry = LocalRegistry::new()?;
    
    // Check if already installed
    if let Some(local_app) = registry.get_app(&uri.app_id).await? {
        if let Some(requested_version) = &uri.version {
            if &local_app.app_info.version == requested_version {
                println!("✅ App {} v{} is already installed!", uri.app_id, requested_version);
                return Ok(());
            } else {
                println!("⚠️  App {} is installed with version {}, but you requested {}",
                    uri.app_id, local_app.app_info.version, requested_version);
            }
        } else {
            println!("✅ App {} v{} is already installed!", uri.app_id, local_app.app_info.version);
            return Ok(());
        }
    }
    
    // Download the package
    println!("📦 Downloading package...");
    let package = store.download(&uri).await?;
    
    println!("✅ Downloaded {} v{} by {}", 
        package.manifest.name, 
        package.manifest.version, 
        package.manifest.developer
    );
    
    // Create installation directory
    let app_dir = registry.apps_dir().join(&uri.app_id.0);
    tokio::fs::create_dir_all(&app_dir).await?;
    
    // Extract package contents
    println!("📁 Extracting package to: {}", app_dir.display());
    extract_package(&package, &app_dir).await?;
    
    // Add to local registry
    let app_info = package.to_app_info();
    registry.add_app(app_info.clone(), app_dir.clone()).await?;
    
    println!("✅ Successfully installed {} v{}!", app_info.name, app_info.version);
    println!("📁 Installed to: {}", app_dir.display());
    
    Ok(())
}

/// Extract a package to the specified directory
async fn extract_package(package: &Package, target_dir: &std::path::Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    use std::io::Cursor;
    
    // Decompress and extract the package content
    let decoder = GzDecoder::new(Cursor::new(&package.content));
    let mut archive = Archive::new(decoder);
    
    // Extract all files
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let target_path = target_dir.join(&path);
        
        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Extract file
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;
        tokio::fs::write(&target_path, &content).await?;
    }
    
    Ok(())
}

async fn cmd_list(verbose: bool) -> Result<()> {
    println!("📱 Installed apps:");
    
    let registry = LocalRegistry::new()?;
    let apps = registry.list_apps().await?;
    
    if apps.is_empty() {
        println!("  No apps installed yet.");
        println!("  Try installing an app with: roselite install veil://app/<app-id>");
        return Ok(());
    }
    
    // Sort by name
    let mut sorted_apps = apps;
    sorted_apps.sort_by(|a, b| a.app_info.name.cmp(&b.app_info.name));
    
    for app in sorted_apps {
        if verbose {
            println!();
            println!("📦 {}", app.app_info.name);
            println!("   Version: {}", app.app_info.version);
            println!("   Developer: {}", app.app_info.developer);
            println!("   Description: {}", app.app_info.description);
            println!("   Category: {}", app.app_info.category);
            println!("   Entry Point: {}", app.app_info.entry_point);
            println!("   Size: {} bytes", app.app_info.size_bytes);
            println!("   Installed: {}", app.installed_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("   Path: {}", app.install_path.display());
            if !app.app_info.tags.is_empty() {
                println!("   Tags: {}", app.app_info.tags.join(", "));
            }
            println!("   URI: {}", app.app_info.uri());
        } else {
            println!("  {} v{} by {} ({})", 
                app.app_info.name, 
                app.app_info.version, 
                app.app_info.developer,
                app.app_info.id
            );
        }
    }
    
    if !verbose {
        println!();
        println!("Use --verbose flag for detailed information");
    }
    
    Ok(())
}

async fn cmd_run(app: String) -> Result<()> {
    println!("🚀 Running app: {}", app);
    
    let registry = LocalRegistry::new()?;
    
    // Find the app by name or ID
    let local_app = registry.find_app_by_name(&app).await?
        .ok_or_else(|| color_eyre::eyre::eyre!("App '{}' not found. Use 'roselite list' to see installed apps", app))?;
    
    println!("📦 Found app: {} v{}", local_app.app_info.name, local_app.app_info.version);
    
    // Check if executable exists
    if !local_app.executable_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Entry point not found: {}\nApp may be corrupted. Try reinstalling.", 
            local_app.executable_path.display()
        ));
    }
    
    // Determine how to run the app based on entry point
    let entry_point = &local_app.app_info.entry_point;
    let working_dir = &local_app.install_path;
    
    println!("📁 Working directory: {}", working_dir.display());
    println!("🎯 Entry point: {}", entry_point);
    
    // Execute the command
    let result = if entry_point.ends_with(".html") || entry_point.ends_with(".htm") {
        // Web app - open in default browser
        println!("🌐 Opening web app in default browser...");
        
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(local_app.executable_path.to_string_lossy().as_ref())
                .current_dir(working_dir)
                .spawn()
        }
        
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(local_app.executable_path.to_string_lossy().as_ref())
                .current_dir(working_dir)
                .spawn()
        }
        
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("start")
                .arg(local_app.executable_path.to_string_lossy().as_ref())
                .current_dir(working_dir)
                .spawn()
        }
    } else if entry_point.ends_with(".py") {
        // Python script
        println!("🐍 Running Python script...");
        std::process::Command::new("python3")
            .arg(&local_app.app_info.entry_point)
            .current_dir(working_dir)
            .spawn()
    } else if entry_point.ends_with(".js") {
        // Node.js script
        println!("📜 Running Node.js script...");
        std::process::Command::new("node")
            .arg(&local_app.app_info.entry_point)
            .current_dir(working_dir)
            .spawn()
    } else {
        // Try to execute directly
        println!("⚙️  Running executable...");
        std::process::Command::new(&local_app.executable_path)
            .current_dir(working_dir)
            .spawn()
    };
    
    // Handle the spawn result
    match result {
        Ok(mut child) => {
            println!("✅ App launched successfully!");
            println!("Press Ctrl+C to return to terminal (app will continue running)");
            
            // Wait for the process to complete or user interrupt
            tokio::select! {
                result = tokio::task::spawn_blocking(move || child.wait()) => {
                    match result? {
                        Ok(status) => {
                            if status.success() {
                                println!("✅ App exited successfully");
                            } else {
                                println!("⚠️  App exited with status: {}", status);
                            }
                        }
                        Err(e) => {
                            println!("❌ App execution failed: {}", e);
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\n🛑 Returning to terminal (app continues in background)");
                }
            }
        }
        Err(e) => {
            return Err(color_eyre::eyre::eyre!(
                "Failed to launch app: {}\n\nTroubleshooting:\n- Ensure the app's runtime is installed\n- Check file permissions\n- Verify entry point exists", 
                e
            ));
        }
    }
    
    Ok(())
}

async fn cmd_search(
    query: String,
    tags: Option<String>,
    developer: Option<String>,
    limit: Option<usize>,
) -> Result<()> {
    println!("🔍 Searching for: {}", query);
    
    let mut filter = types::SearchFilter::default();
    filter.query = Some(query);
    filter.limit = limit;
    
    if let Some(tags_str) = tags {
        filter.tags = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
    }
    
    if let Some(dev) = developer {
        filter.developer = Some(dev);
    }
    
    let store = store::VeilidStore::new().await?;
    let results = store.search(&filter).await?;
    
    println!("📦 Found {} apps:", results.len());
    for app in results {
        println!("  {} v{} by {}", app.name, app.version, app.developer);
        if !app.description.is_empty() {
            println!("    {}", app.description);
        }
        println!("    🔗 {}", app.uri());
        println!();
    }
    
    Ok(())
}

async fn cmd_info(app: String) -> Result<()> {
    println!("ℹ️  App information for: {}", app);
    
    // Try to parse as URI first
    if app.starts_with("veil://") {
        return cmd_info_from_uri(app).await;
    }
    
    // Otherwise, look for locally installed app
    let registry = LocalRegistry::new()?;
    
    if let Some(local_app) = registry.find_app_by_name(&app).await? {
        println!();
        println!("📦 {} (Installed)", local_app.app_info.name);
        println!("═══════════════════════════════════════");
        println!("🆔 App ID: {}", local_app.app_info.id);
        println!("📈 Version: {}", local_app.app_info.version);
        println!("👨‍💻 Developer: {}", local_app.app_info.developer);
        println!("📂 Category: {}", local_app.app_info.category);
        println!("📝 Description:");
        println!("   {}", local_app.app_info.description);
        println!("🎯 Entry Point: {}", local_app.app_info.entry_point);
        println!("📏 Size: {} bytes", local_app.app_info.size_bytes);
        
        if !local_app.app_info.tags.is_empty() {
            println!("🏷️  Tags: {}", local_app.app_info.tags.join(", "));
        }
        
        println!("📅 Created: {}", local_app.app_info.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("🔄 Updated: {}", local_app.app_info.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("📅 Installed: {}", local_app.installed_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("📁 Install Path: {}", local_app.install_path.display());
        println!("🔗 URI: {}", local_app.app_info.uri());
        
        if let Some(signature) = &local_app.app_info.signature {
            println!("🔐 Signature: {}...", &signature[..signature.len().min(16)]);
        }
        
        println!();
        println!("Commands:");
        println!("  🚀 Run:       roselite run {}", local_app.app_info.name);
        println!("  🗑️  Uninstall: roselite uninstall {} (not implemented)", local_app.app_info.name);
        
    } else {
        // Try to fetch from store
        let store = store::VeilidStore::new().await?;
        let app_id = types::AppId(app.clone());
        
        match store.get_app(&app_id).await? {
            Some(app_info) => {
                println!();
                println!("📦 {} (Available for Install)", app_info.name);
                println!("═══════════════════════════════════════");
                println!("🆔 App ID: {}", app_info.id);
                println!("📈 Version: {}", app_info.version);
                println!("👨‍💻 Developer: {}", app_info.developer);
                println!("📂 Category: {}", app_info.category);
                println!("📝 Description:");
                println!("   {}", app_info.description);
                println!("🎯 Entry Point: {}", app_info.entry_point);
                println!("📏 Size: {} bytes", app_info.size_bytes);
                println!("⭐ Rating: {:.1}/5.0", app_info.rating);
                println!("📥 Downloads: {}", app_info.download_count);
                
                if !app_info.tags.is_empty() {
                    println!("🏷️  Tags: {}", app_info.tags.join(", "));
                }
                
                println!("📅 Created: {}", app_info.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("🔄 Updated: {}", app_info.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("🔗 URI: {}", app_info.uri());
                
                if let Some(signature) = &app_info.signature {
                    println!("🔐 Signature: {}...", &signature[..signature.len().min(16)]);
                }
                
                println!();
                println!("Commands:");
                println!("  📥 Install: roselite install {}", app_info.uri());
                
            }
            None => {
                println!("❌ App '{}' not found locally or in the store", app);
                println!("💡 Try searching: roselite search {}", app);
            }
        }
    }
    
    Ok(())
}

async fn cmd_info_from_uri(uri_str: String) -> Result<()> {
    let uri = parse_veil_uri(&uri_str)?;
    let store = store::VeilidStore::new().await?;
    
    // Try to get from store
    match store.get_app(&uri.app_id).await? {
        Some(app_info) => {
            println!();
            println!("📦 {}", app_info.name);
            println!("═══════════════════════════════════════");
            println!("🆔 App ID: {}", app_info.id);
            println!("📈 Version: {}", app_info.version);
            println!("👨‍💻 Developer: {}", app_info.developer);
            println!("📂 Category: {}", app_info.category);
            println!("📝 Description:");
            println!("   {}", app_info.description);
            println!("🎯 Entry Point: {}", app_info.entry_point);
            println!("📏 Size: {} bytes", app_info.size_bytes);
            println!("⭐ Rating: {:.1}/5.0", app_info.rating);
            println!("📥 Downloads: {}", app_info.download_count);
            
            if !app_info.tags.is_empty() {
                println!("🏷️  Tags: {}", app_info.tags.join(", "));
            }
            
            println!("📅 Created: {}", app_info.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("🔄 Updated: {}", app_info.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("🔗 URI: {}", uri_str);
            
            if let Some(signature) = &app_info.signature {
                println!("🔐 Signature: {}...", &signature[..signature.len().min(16)]);
            }
            
            // Check if installed
            let registry = LocalRegistry::new()?;
            if let Some(_local_app) = registry.get_app(&uri.app_id).await? {
                println!("✅ Status: Installed");
                println!();
                println!("Commands:");
                println!("  🚀 Run: roselite run {}", app_info.name);
            } else {
                println!("📦 Status: Available for Install");
                println!();
                println!("Commands:");
                println!("  📥 Install: roselite install {}", uri_str);
            }
        }
        None => {
            println!("❌ App not found at URI: {}", uri_str);
        }
    }
    
    Ok(())
} 