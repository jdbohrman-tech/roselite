use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::*,
};
use roselite_core::*;
use std::io::{stdout, Stdout};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    
    tracing_subscriber::fmt()
        .with_env_filter("roselite_store=info,warn")
        .init();

    let mut app = App::new().await?;
    let mut terminal = setup_terminal()?;
    
    let result = run_app(&mut app, &mut terminal).await;
    
    restore_terminal(&mut terminal)?;
    
    result
}

fn setup_terminal() -> Result<Terminal> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal) -> Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_app(app: &mut App, terminal: &mut Terminal) -> Result<()> {
    loop {
        terminal.draw(|frame| app.draw(frame))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.handle_key(key.code).await {
                    Some(AppAction::Quit) => break,
                    Some(AppAction::Search) => {
                        app.perform_search().await?;
                    }
                    Some(AppAction::Install) => {
                        app.install_selected().await?;
                    }
                    None => {}
                }
            }
        }
    }
    Ok(())
}

enum AppAction {
    Quit,
    Search,
    Install,
}

enum InputMode {
    Normal,
    Search,
}

struct App {
    store: store::VeilidStore,
    input_mode: InputMode,
    search_query: String,
    cursor_position: usize,
    apps: Vec<types::AppInfo>,
    filtered_apps: Vec<(types::AppInfo, i64)>, // (app, score)
    selected_index: usize,
    matcher: SkimMatcherV2,
    status_message: String,
}

impl App {
    async fn new() -> Result<Self> {
        let store = store::VeilidStore::new().await?;
        
        Ok(Self {
            store,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            cursor_position: 0,
            apps: Vec::new(),
            filtered_apps: Vec::new(),
            selected_index: 0,
            matcher: SkimMatcherV2::default(),
            status_message: "Press '/' to search, 'q' to quit".to_string(),
        })
    }

    async fn handle_key(&mut self, key: KeyCode) -> Option<AppAction> {
        match self.input_mode {
            InputMode::Normal => match key {
                KeyCode::Char('q') => Some(AppAction::Quit),
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Search;
                    self.search_query.clear();
                    self.cursor_position = 0;
                    None
                }
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    None
                }
                KeyCode::Down => {
                    if self.selected_index < self.filtered_apps.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                    None
                }
                KeyCode::Enter => Some(AppAction::Install),
                _ => None,
            },
            InputMode::Search => match key {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    None
                }
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    Some(AppAction::Search)
                }
                KeyCode::Char(c) => {
                    self.search_query.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    self.filter_apps();
                    None
                }
                KeyCode::Backspace => {
                    if self.cursor_position > 0 {
                        self.search_query.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                        self.filter_apps();
                    }
                    None
                }
                KeyCode::Left => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                    }
                    None
                }
                KeyCode::Right => {
                    if self.cursor_position < self.search_query.len() {
                        self.cursor_position += 1;
                    }
                    None
                }
                _ => None,
            },
        }
    }

    async fn perform_search(&mut self) -> Result<()> {
        self.status_message = "Searching Veilid DHT...".to_string();
        
        let filter = types::SearchFilter {
            query: if self.search_query.is_empty() { None } else { Some(self.search_query.clone()) },
            ..Default::default()
        };
        
        match self.store.search(&filter).await {
            Ok(apps) => {
                self.apps = apps;
                self.filter_apps();
                self.status_message = format!("Found {} apps", self.apps.len());
            }
            Err(e) => {
                self.status_message = format!("Search failed: {}", e);
            }
        }
        
        Ok(())
    }

    fn filter_apps(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_apps = self.apps
                .iter()
                .map(|app| (app.clone(), 100))
                .collect();
        } else {
            self.filtered_apps = self.apps
                .iter()
                .filter_map(|app| {
                    let name_score = self.matcher.fuzzy_match(&app.name, &self.search_query);
                    let desc_score = self.matcher.fuzzy_match(&app.description, &self.search_query);
                    let dev_score = self.matcher.fuzzy_match(&app.developer, &self.search_query);
                    
                    let best_score = [name_score, desc_score, dev_score]
                        .iter()
                        .filter_map(|&s| s)
                        .max()
                        .unwrap_or(0);
                    
                    if best_score > 0 {
                        Some((app.clone(), best_score))
                    } else {
                        None
                    }
                })
                .collect();
                
            // Sort by score (highest first)
            self.filtered_apps.sort_by(|a, b| b.1.cmp(&a.1));
        }
        
        self.selected_index = 0;
    }

    async fn install_selected(&mut self) -> Result<()> {
        if let Some((app, _)) = self.filtered_apps.get(self.selected_index).cloned() {
            self.status_message = format!("Installing {}...", app.name);
            
            match self.install_app(&app).await {
                Ok(()) => {
                    self.status_message = format!("✅ {} installed successfully!", app.name);
                }
                Err(e) => {
                    self.status_message = format!("❌ Failed to install {}: {}", app.name, e);
                }
            }
        }
        
        Ok(())
    }

    async fn install_app(&mut self, app: &types::AppInfo) -> Result<()> {
        // Create URI for download
        let uri = app.uri();
        
        // Download the package
        let package = self.store.download(&uri).await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to download package: {}", e))?;
        
        // Verify the package signature
        let crypto = roselite_core::crypto::CryptoManager::new()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to initialize crypto: {}", e))?;
        
        let is_valid = package.verify_signature(&crypto)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to verify signature: {}", e))?;
        
        if !is_valid {
            return Err(color_eyre::eyre::eyre!("Package signature verification failed"));
        }
        
        // Create installation directory
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| color_eyre::eyre::eyre!("Could not determine home directory"))?;
        
        let apps_dir = std::path::Path::new(&home_dir)
            .join(".roselite")
            .join("apps")
            .join(&app.id.0);
        
        // Remove existing installation if it exists
        if apps_dir.exists() {
            tokio::fs::remove_dir_all(&apps_dir).await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to remove existing installation: {}", e))?;
        }
        
        // Create installation directory
        tokio::fs::create_dir_all(&apps_dir).await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create installation directory: {}", e))?;
        
        // Extract package content
        use flate2::read::GzDecoder;
        use tar::Archive;
        use std::io::Cursor;
        
        let cursor = Cursor::new(&package.content);
        let decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(decoder);
        
        // Extract all files
        archive.unpack(&apps_dir)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to extract package: {}", e))?;
        
        // Save package metadata
        let metadata_file = apps_dir.join(".roselite-metadata.json");
        let metadata = serde_json::to_string_pretty(&package.manifest)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to serialize metadata: {}", e))?;
        
        tokio::fs::write(&metadata_file, metadata).await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write metadata: {}", e))?;
        
        // Update installed apps registry
        self.update_installed_registry(app, &apps_dir).await?;
        
        Ok(())
    }

    async fn update_installed_registry(&self, app: &types::AppInfo, install_path: &std::path::Path) -> Result<()> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| color_eyre::eyre::eyre!("Could not determine home directory"))?;
        
        let registry_file = std::path::Path::new(&home_dir)
            .join(".roselite")
            .join("installed.json");
        
        // Ensure .roselite directory exists
        if let Some(parent) = registry_file.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| color_eyre::eyre::eyre!("Failed to create roselite directory: {}", e))?;
        }
        
        // Load existing registry or create new one
        let mut installed_apps: std::collections::HashMap<String, serde_json::Value> = 
            if registry_file.exists() {
                let content = tokio::fs::read_to_string(&registry_file).await
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to read registry: {}", e))?;
                serde_json::from_str(&content)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to parse registry: {}", e))?
            } else {
                std::collections::HashMap::new()
            };
        
        // Add/update app entry
        let app_entry = serde_json::json!({
            "id": app.id.0,
            "name": app.name,
            "version": app.version,
            "developer": app.developer,
            "install_path": install_path.to_string_lossy(),
            "installed_at": chrono::Utc::now().to_rfc3339(),
            "entry_point": app.entry_point
        });
        
        installed_apps.insert(app.id.0.clone(), app_entry);
        
        // Save updated registry
        let registry_content = serde_json::to_string_pretty(&installed_apps)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to serialize registry: {}", e))?;
        
        tokio::fs::write(&registry_file, registry_content).await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write registry: {}", e))?;
        
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let main_layout = Layout::vertical([
            Constraint::Length(3), // Search bar
            Constraint::Min(0),    // App list
            Constraint::Length(3), // Status/help
        ]);
        
        let [search_area, list_area, status_area] = main_layout.areas(frame.area());

        self.draw_search_bar(frame, search_area);
        self.draw_app_list(frame, list_area);
        self.draw_status_bar(frame, status_area);
    }

    fn draw_search_bar(&self, frame: &mut Frame, area: Rect) {
        let search_text = if matches!(self.input_mode, InputMode::Search) {
            format!("🔍 {}", self.search_query)
        } else {
            format!("🔍 {} (Press '/' to search)", self.search_query)
        };

        let search_widget = Paragraph::new(search_text)
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Search => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Search"));

        frame.render_widget(search_widget, area);

        // Show cursor in search mode
        if matches!(self.input_mode, InputMode::Search) {
            frame.set_cursor_position(Position::new(
                area.x + self.cursor_position as u16 + 3, // Account for border and emoji
                area.y + 1,
            ));
        }
    }

    fn draw_app_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.filtered_apps
            .iter()
            .enumerate()
            .map(|(i, (app, score))| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let content = vec![
                    Line::from(vec![
                        Span::styled(format!("📦 {}", app.name), Style::default().bold()),
                        Span::raw(format!(" v{}", app.version)),
                        if self.search_query.is_empty() {
                            Span::raw("")
                        } else {
                            Span::styled(format!(" ({})", score), Style::default().dim())
                        },
                    ]),
                    Line::from(vec![
                        Span::styled(format!("👤 {}", app.developer), Style::default().italic()),
                        Span::raw(format!(" | 📦 {} bytes", app.size_bytes)),
                    ]),
                    Line::from(Span::raw(format!("📝 {}", app.description))),
                ];

                ListItem::new(content).style(style)
            })
            .collect();

        let list_widget = List::new(items)
            .block(Block::bordered().title(format!("Apps ({})", self.filtered_apps.len())))
            .highlight_style(Style::default().bg(Color::Blue))
            .highlight_symbol("► ");

        frame.render_widget(list_widget, area);
    }

    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status_text = match self.input_mode {
            InputMode::Normal => format!(
                "{} | ↑↓ Navigate | Enter Install | / Search | q Quit",
                self.status_message
            ),
            InputMode::Search => format!(
                "{} | Esc Cancel | Enter Search",
                self.status_message
            ),
        };

        let status_widget = Paragraph::new(status_text)
            .style(Style::default().dim())
            .block(Block::bordered().title("Status"));

        frame.render_widget(status_widget, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_install_app_logic() {
        // Create a mock app info
        let app = types::AppInfo {
            id: types::AppId("test-app".to_string()),
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            description: "A test application".to_string(),
            developer: "Test Developer".to_string(),
            category: "test".to_string(),
            size_bytes: 1024,
            download_count: 0,
            rating: 5.0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: vec!["test".to_string()],
            entry_point: "index.html".to_string(),
            veilid_identity: None,
            signature: None,
        };

        // Test URI creation
        let uri = app.uri();
        assert_eq!(uri.app_id.0, "test-app");
        assert_eq!(uri.version, Some("1.0.0".to_string()));

        // Test that our installation directory logic is sound
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        
        let apps_dir = temp_dir.path()
            .join(".roselite")
            .join("apps")
            .join(&app.id.0);
        
        assert_eq!(apps_dir.file_name().unwrap(), "test-app");
        
        // Verify registry file path
        let registry_file = temp_dir.path()
            .join(".roselite")
            .join("installed.json");
        
        assert_eq!(registry_file.file_name().unwrap(), "installed.json");
    }
} 