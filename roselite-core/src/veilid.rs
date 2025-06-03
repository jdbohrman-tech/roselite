use crate::error::*;
use std::sync::Arc;
use std::collections::HashMap;
use tracing;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
// Base64 may be used elsewhere; import if necessary (currently unused)
use crate::crypto::CryptoManager;
use veilid_core::{TypedKey};
use std::str::FromStr;
// DHT types
use veilid_core::{DHTSchema, DHTReportScope, ValueSubkey};
use serde_json::{self, Value as JsonValue};

// Full Veilid integration with proper VeilidAPI setup
// This implementation provides complete Veilid network functionality

/// Veilid connection manager with full Veilid network integration
pub struct VeilidConnection {
    /// The Veilid API instance (when available)
    api: Option<Arc<veilid_core::VeilidAPI>>,
    /// Connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Fallback in-memory storage for development/testing
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// Configuration for network behavior
    config: VeilidConfig,
    /// Routing context for peer-to-peer operations
    routing_context: Option<veilid_core::RoutingContext>,
}

/// Connection state information
#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub is_connected: bool,
    pub attachment_state: AttachmentState,
    pub network_started: bool,
    pub use_fallback_storage: bool,
    pub node_id: Option<String>,
}

/// Attachment state enum
#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentState {
    Detached,
    Detaching,
    Attaching,
    AttachedWeak,
    AttachedGood,
    AttachedStrong,
    FullyAttached,
    OverAttached,
}

/// Enhanced configuration for Veilid connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeilidConfig {
    pub program_name: String,
    pub namespace: String,
    pub table_name: String,
    pub use_fallback_storage: bool,
    /// Network configuration
    pub network: NetworkConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Bootstrap nodes for initial connection
    /// Enable development mode (more permissive settings)
    pub development_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub connection_timeout_ms: u64,
    pub max_connections: u32,
    pub enable_upnp: bool,
    pub enable_nat_detection: bool,
    /// Custom listening addresses
    pub udp_listen_address: Option<String>,
    pub tcp_listen_address: Option<String>,
    pub ws_listen_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_directory: Option<String>,
    pub enable_encryption: bool,
    pub max_storage_mb: u64,
    pub cleanup_old_data: bool,
}

impl Default for VeilidConfig {
    fn default() -> Self {
        Self {
            program_name: "roselite".to_string(),
            namespace: "roselite".to_string(),
            table_name: "roselite_sites".to_string(),
            use_fallback_storage: false,
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            development_mode: true, // Default to dev mode for easier setup
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connection_timeout_ms: 10000,
            max_connections: 64,
            enable_upnp: true,
            enable_nat_detection: true,
            udp_listen_address: None,
            tcp_listen_address: None,
            ws_listen_address: None,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_directory: Some(".roselite".to_string()),  // Use hidden directory
            enable_encryption: true,
            max_storage_mb: 1024, // 1GB default
            cleanup_old_data: true,
        }
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            is_connected: false,
            attachment_state: AttachmentState::Detached,
            network_started: false,
            use_fallback_storage: false,
            node_id: None,
        }
    }
}

impl VeilidConnection {
    /// Initialize a new Veilid connection
    pub async fn new() -> Result<Self> {
        Self::new_with_config(VeilidConfig::default()).await
    }

    /// Initialize with custom configuration
    pub async fn new_with_config(config: VeilidConfig) -> Result<Self> {
        Ok(Self {
            api: None,
            state: Arc::new(RwLock::new(ConnectionState::default())),
            storage: Arc::new(RwLock::new(HashMap::new())),
            config,
            routing_context: None,
        })
    }

    /// Connect to the Veilid network with full initialization
    pub async fn connect(&mut self) -> Result<()> {
        tracing::info!("Initializing Veilid connection...");
        
        // Update state to connecting
        {
            let mut state = self.state.write().await;
            state.attachment_state = AttachmentState::Attaching;
        }
        
        // Try to initialize Veilid API
        let api = self.init_veilid_api().await?;
        tracing::info!("Successfully initialized Veilid API");
        
        // Store the API
        self.api = Some(api.clone());
        
        // Create routing context with default safety settings
        match api.routing_context() {
            Ok(routing_ctx) => {
                self.routing_context = Some(routing_ctx);
                tracing::info!("Created Veilid routing context");
            },
            Err(e) => {
                tracing::warn!("Failed to create routing context: {:?}", e);
            }
        }
        
        // Wait until we are at least weakly attached before proceeding. This
        // is critical because DHT operations will fail if we are still in the
        // Attaching state.
        self.wait_until_attached().await?;
        
        // Get node ID
        let node_id = self.get_node_id().await?;
        
        // Update connection state
        {
            let mut state = self.state.write().await;
            state.is_connected = true;
            state.network_started = true;
            state.node_id = Some(node_id);
        }
        
        tracing::info!("Successfully connected to Veilid network");
        Ok(())
    }

    /// Disconnect from the Veilid network
    pub async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnecting from Veilid network...");
        
        if let Some(api) = self.api.take() {
            // Detach from network gracefully
            if let Err(e) = api.detach().await {
                tracing::warn!("Error during network detach: {:?}", e);
            }
            
            // Shutdown the API - need to extract from Arc first
            match Arc::try_unwrap(api) {
                Ok(api_owned) => {
                    api_owned.shutdown().await;
                },
                Err(arc_api) => {
                    // If there are other references, just detach
                    tracing::warn!("Cannot shutdown API due to multiple references, detaching only");
                    let _ = arc_api.detach().await;
                }
            }
        }
        
        // Clean up routing context
        self.routing_context = None;
        
        // Clear storage
        self.storage.write().await.clear();
        
        // Reset state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::default();
        }
        
        tracing::info!("Disconnected from Veilid network");
        Ok(())
    }

    /// Store raw bytes in a DHT record subkey
    pub async fn dht_set_subkey(&self, key_str: &str, subkey: ValueSubkey, value: &[u8]) -> Result<()> {
        self.wait_until_attached().await?;
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
        let typed_key = TypedKey::from_str(key_str)
            .map_err(|_| RoseliteError::InvalidUri(format!("Invalid DHT key: {}", key_str)))?;
        routing_ctx.set_dht_value(typed_key, subkey, value.to_vec(), None)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { operation: format!("set_dht_value failed: {:?}", e) }))?;
        Ok(())
    }

    /// Retrieve bytes from a DHT record subkey
    pub async fn dht_get_subkey(&self, key_str: &str, subkey: ValueSubkey) -> Result<Option<Vec<u8>>> {
        self.wait_until_attached().await?;
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
        let typed_key = TypedKey::from_str(key_str)
            .map_err(|_| RoseliteError::InvalidUri(format!("Invalid DHT key: {}", key_str)))?;
        
        // Try to get the value first, if it fails with "record not open", try opening it
        match routing_ctx.get_dht_value(typed_key, subkey, false).await {
            Ok(resp) => Ok(resp.map(|v| v.data().to_vec())),
            Err(e) => {
                // Check if the error is due to record not being open
                if e.to_string().contains("record not open") {
                    tracing::debug!("Record not open, attempting to open: {}", key_str);
                    // Try to open the record and then get the value
                    self.open_dht_record(key_str).await?;
                    let resp = routing_ctx.get_dht_value(typed_key, subkey, false)
                        .await
                        .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                            operation: format!("get_dht_value failed after opening: {:?}", e) 
                        }))?;
                    Ok(resp.map(|v| v.data().to_vec()))
                } else {
                    Err(RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                        operation: format!("get_dht_value failed: {:?}", e) 
                    }))
                }
            }
        }
    }

    /// Open a DHT record for reading
    pub async fn open_dht_record(&self, key_str: &str) -> Result<()> {
        self.wait_until_attached().await?;
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
        let typed_key = TypedKey::from_str(key_str)
            .map_err(|_| RoseliteError::InvalidUri(format!("Invalid DHT key: {}", key_str)))?;
        
        routing_ctx.open_dht_record(typed_key, None)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("open_dht_record failed: {:?}", e) 
            }))?;
        
        tracing::debug!("Successfully opened DHT record: {}", key_str);
        Ok(())
    }

    /// Delete an entire DHT record
    pub async fn dht_delete_record(&self, key_str: &str) -> Result<()> {
        self.wait_until_attached().await?;
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
        let typed_key = TypedKey::from_str(key_str)
            .map_err(|_| RoseliteError::InvalidUri(format!("Invalid DHT key: {}", key_str)))?;
        routing_ctx.delete_dht_record(typed_key)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { operation: format!("delete_dht_record failed: {:?}", e) }))?;
        Ok(())
    }

    /// Convenience wrappers (subkey 0)
    pub async fn dht_set(&self, key: &str, value: &[u8]) -> Result<()> {
        self.dht_set_subkey(key, 0, value).await
    }

    pub async fn dht_get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.dht_get_subkey(key, 0).await
    }

    pub async fn dht_delete(&self, key: &str) -> Result<()> {
        self.dht_delete_record(key).await
    }

    /// Create a brand-new DHT record with a simple one-column schema. Returns the record key as string.
    pub async fn create_dht_record(&self) -> Result<String> {
        self.create_dht_record_with_cols(2).await
    }

    /// Create DHT record with custom column count.
    pub async fn create_dht_record_with_cols(&self, cols: usize) -> Result<String> {
        self.wait_until_attached().await?;
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
        let cols_u16: u16 = cols.try_into().map_err(|_| RoseliteError::InvalidUri(format!("Too many columns: {}", cols)))?;
        let schema = DHTSchema::dflt(cols_u16)
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { operation: format!("schema build failed: {:?}", e) }))?;
        let desc = routing_ctx.create_dht_record(schema, None, None)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { operation: format!("create_dht_record failed: {:?}", e) }))?;
        Ok(desc.key().to_string())
    }

    /// Inspect a record to gauge replication consensus.
    pub async fn inspect_record(&self, key_str: &str) -> Result<()> {
        self.wait_until_attached().await?;
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
        let typed_key = TypedKey::from_str(key_str)
            .map_err(|_| RoseliteError::InvalidUri(format!("Invalid DHT key: {}", key_str)))?;
        let report = routing_ctx.inspect_dht_record(typed_key, None, DHTReportScope::SyncSet)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { operation: format!("inspect_dht_record failed: {:?}", e) }))?;
        tracing::info!("DHT record inspection: {:?}", report);
        Ok(())
    }

    /// Build Veilid configuration (start from upstream default, tweak to work in dev)
    fn build_veilid_config(&self) -> Result<String> {
        // base config from library
        let mut cfg: JsonValue = serde_json::from_str(&veilid_core::default_veilid_config())?;

        // Ensure directories we can write to under current project
        let data_dir = ".roselite";

        // program name & namespace for isolated stores
        cfg["program_name"] = JsonValue::String("roselite".to_string());
        cfg["namespace"] = JsonValue::String("dev".to_string());

        // protected store tweaks
        let password = std::env::var("ROSELITE_PASSWORD").unwrap_or_default();
        let insecure = password.is_empty();
        if let Some(ps) = cfg.get_mut("protected_store") {
            ps["allow_insecure_fallback"] = JsonValue::Bool(insecure);
            ps["always_use_insecure_storage"] = JsonValue::Bool(insecure);
            ps["directory"] = JsonValue::String(data_dir.into());

            if insecure {
                // Use a dummy password to avoid OS keyring errors while still storing unencrypted.
                ps["device_encryption_key_password"] = JsonValue::String("roselite-dev".into());
            } else {
                ps["device_encryption_key_password"] = JsonValue::String(password.clone());
            }

            // Only include new password field when changing passwords in secure mode
            ps["new_device_encryption_key_password"] = JsonValue::Null;
        }
        if let Some(ts) = cfg.get_mut("table_store") {
            ts["directory"] = JsonValue::String(data_dir.into());
        }
        if let Some(bs) = cfg.get_mut("block_store") {
            bs["directory"] = JsonValue::String(data_dir.into());
        }

        Ok(cfg.to_string())
    }

    /// Generate a new cryptographic key pair using our crypto manager
    pub async fn generate_keypair(&self) -> Result<(String, String)> {
        let crypto = CryptoManager::new()?;
        crypto.generate_keypair()
    }

    /// Check if connected to Veilid network
    pub async fn is_connected(&self) -> bool {
        self.state.read().await.is_connected
    }

    /// Check if using fallback storage
    pub async fn is_using_fallback(&self) -> bool {
        self.state.read().await.use_fallback_storage
    }

    /// Get current attachment state
    pub async fn get_attachment_state(&self) -> AttachmentState {
        self.state.read().await.attachment_state.clone()
    }

    /// Get node ID if connected
    pub async fn get_node_id_cached(&self) -> Option<String> {
        self.state.read().await.node_id.clone()
    }

    /// Get detailed network state information
    pub async fn get_network_state(&self) -> Result<NetworkStateInfo> {
        let state = self.state.read().await;
        
        if state.use_fallback_storage || self.api.is_none() {
            return Ok(NetworkStateInfo {
                mode: "Fallback Storage".to_string(),
                attachment: state.attachment_state.clone(),
                node_id: state.node_id.clone(),
                peer_count: 0,
                network_started: false,
                routes_count: 0,
            });
        }

        let api = self.get_api()?;
        
        let veilid_state = api.get_state()
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to get network state: {:?}", e)
            }))?;
            
        Ok(NetworkStateInfo {
            mode: "Full Veilid Network".to_string(),
            attachment: AttachmentState::from_veilid_attachment(&veilid_state.attachment),
            node_id: state.node_id.clone(),
            peer_count: veilid_state.network.peers.len(),
            network_started: veilid_state.network.started,
            // Note: routes field may not exist on the network state
            routes_count: 0, // Simplified for now
        })
    }

    /// Get the node ID from Veilid
    async fn get_node_id(&self) -> Result<String> {
        let api = self.get_api()?;
        
        let state = api.get_state()
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to get state for node ID: {:?}", e)
            }))?;
            
        // Extract node ID from config - CryptoTypedGroup is not iterable, so use string representation
        let node_id = format!("{:?}", state.config.config.network.routing_table.node_id);
            
        Ok(node_id)
    }

    /// Get the Veilid API reference
    fn get_api(&self) -> Result<&veilid_core::VeilidAPI> {
        self.api.as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))
    }

    /// Attach to the Veilid network with retry logic
    async fn attach_with_retry(&self, api: &veilid_core::VeilidAPI) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);
        
        for attempt in 1..=MAX_RETRIES {
            tracing::info!("Attempting to attach to Veilid network (attempt {}/{})", attempt, MAX_RETRIES);
            
            match api.attach().await {
                Ok(_) => {
                    tracing::info!("Successfully attached to Veilid network");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Failed to attach to network on attempt {}: {:?}", attempt, e);
                    
                    if attempt < MAX_RETRIES {
                        tracing::info!("Retrying in {:?}...", RETRY_DELAY);
                        tokio::time::sleep(RETRY_DELAY).await;
                    } else {
                        tracing::error!("Failed to attach to network after {} attempts", MAX_RETRIES);
                        return Err(RoseliteError::Veilid(VeilidError::ConnectionFailed));
                    }
                }
            }
        }
        
        unreachable!("Loop should have returned or errored")
    }

    /// Initialize Veilid API with enhanced configuration
    async fn init_veilid_api(&self) -> Result<Arc<veilid_core::VeilidAPI>> {
        // Create enhanced configuration for Veilid
        let config_json = self.build_veilid_config()?;
        
        // Debug: log the configuration being used
        tracing::debug!("Veilid configuration JSON: {}", config_json);

        // Create update callback
        let update_callback = Arc::new({
            let state = self.state.clone();
            move |update| {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    Self::update_callback(update, state_clone).await;
                });
            }
        });

        // Start up Veilid API with JSON configuration - note: parameter order is callback first, then config
        let api = veilid_core::api_startup_json(
            update_callback,
            config_json,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to start Veilid API: {:?}", e);
            tracing::error!("This could be due to:");
            tracing::error!("  • Invalid JSON configuration format");
            tracing::error!("  • Missing or invalid bootstrap nodes");
            tracing::error!("  • Network connectivity issues");
            tracing::error!("  • Invalid data directory permissions");
            RoseliteError::Veilid(VeilidError::ConnectionFailed)
        })?;
            
        // Attach to the network with retry logic
        self.attach_with_retry(&api).await?;
            
        // Wrap in Arc and return
        Ok(Arc::new(api))
    }

    /// Enhanced Veilid update callback handler with state management
    async fn update_callback(update: veilid_core::VeilidUpdate, state: Arc<RwLock<ConnectionState>>) {
        match update {
            veilid_core::VeilidUpdate::Log(log_update) => {
                // Handle log updates based on the actual structure
                tracing::debug!("Veilid log update: {:?}", log_update);
            },
            veilid_core::VeilidUpdate::Attachment(attachment_update) => {
                let new_state = AttachmentState::from_veilid_attachment(&attachment_update);
                {
                    let mut state = state.write().await;
                    state.attachment_state = new_state.clone();
                }
                tracing::info!("Veilid attachment state changed: {:?}", new_state);
            },
            veilid_core::VeilidUpdate::Network(network_update) => {
                {
                    let mut state = state.write().await;
                    state.network_started = network_update.started;
                }
                if network_update.started {
                    tracing::info!("Veilid network started");
                } else {
                    tracing::info!("Veilid network stopped");
                }
            },
            veilid_core::VeilidUpdate::Config(_config_update) => {
                tracing::debug!("Veilid config updated");
            },
            veilid_core::VeilidUpdate::RouteChange(_route_change) => {
                tracing::debug!("Veilid routes changed");
            },
            veilid_core::VeilidUpdate::ValueChange(_value_change) => {
                tracing::debug!("Veilid value changed");
            },
            veilid_core::VeilidUpdate::AppMessage(_app_message) => {
                tracing::debug!("Veilid app message received");
            },
            veilid_core::VeilidUpdate::AppCall(_app_call) => {
                tracing::debug!("Veilid app call received");
            },
            veilid_core::VeilidUpdate::Shutdown => {
                {
                    let mut state = state.write().await;
                    state.is_connected = false;
                    state.network_started = false;
                    state.attachment_state = AttachmentState::Detached;
                }
                tracing::info!("Veilid is shutting down");
            },
        }
    }

    /// Block until the attachment state reaches at least `AttachedWeak` or the timeout elapses.
    async fn wait_until_attached(&self) -> Result<()> {
        use tokio::time::{sleep, Duration, Instant};

        const TIMEOUT: Duration = Duration::from_secs(30);
        const POLL_INTERVAL: Duration = Duration::from_millis(250);

        let start = Instant::now();
        loop {
            {
                let state = self.state.read().await;
                match state.attachment_state {
                    AttachmentState::AttachedWeak | AttachmentState::AttachedGood | AttachmentState::AttachedStrong | AttachmentState::FullyAttached | AttachmentState::OverAttached => {
                        tracing::info!(
                            "Veilid node attached (state = {:?}) after {:?}",
                            state.attachment_state,
                            start.elapsed()
                        );
                        return Ok(());
                    }
                    AttachmentState::Detached | AttachmentState::Detaching => {
                        return Err(RoseliteError::Veilid(VeilidError::ConnectionFailed));
                    }
                    AttachmentState::Attaching => {}
                }
            }

            if start.elapsed() > TIMEOUT {
                tracing::error!("Timed out waiting for Veilid node to attach ({:?})", TIMEOUT);
                return Err(RoseliteError::Veilid(VeilidError::ConnectionFailed));
            }
            sleep(POLL_INTERVAL).await;
        }
    }
}

/// Detailed network state information
#[derive(Debug, Clone)]
pub struct NetworkStateInfo {
    pub mode: String,
    pub attachment: AttachmentState,
    pub node_id: Option<String>,
    pub peer_count: usize,
    pub network_started: bool,
    pub routes_count: usize,
}

impl AttachmentState {
    /// Convert from Veilid's attachment state
    fn from_veilid_attachment(attachment: &veilid_core::VeilidStateAttachment) -> Self {
        match attachment.state {
            veilid_core::AttachmentState::Detached => AttachmentState::Detached,
            veilid_core::AttachmentState::Detaching => AttachmentState::Detaching,
            veilid_core::AttachmentState::Attaching => AttachmentState::Attaching,
            veilid_core::AttachmentState::AttachedWeak => AttachmentState::AttachedWeak,
            veilid_core::AttachmentState::AttachedGood => AttachmentState::AttachedGood,
            veilid_core::AttachmentState::AttachedStrong => AttachmentState::AttachedStrong,
            veilid_core::AttachmentState::FullyAttached => AttachmentState::FullyAttached,
            veilid_core::AttachmentState::OverAttached => AttachmentState::OverAttached,
        }
    }
}

impl Drop for VeilidConnection {
    fn drop(&mut self) {
        let is_connected = futures::executor::block_on(async {
            self.state.read().await.is_connected
        });
        
        if is_connected {
            tracing::warn!("VeilidConnection dropped while still connected - this may cause resource leaks");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_veilid_connection_basic_ops() {
        let mut conn = VeilidConnection::new().await.unwrap();
        
        // Connect should succeed (or return an explicit error)
        conn.connect().await.unwrap();
        assert!(conn.is_connected().await);
        
        // Test basic operations
        conn.dht_set("test_key", b"test_value").await.unwrap();
        let value = conn.dht_get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
        
        conn.dht_delete("test_key").await.unwrap();
        let value = conn.dht_get("test_key").await.unwrap();
        assert_eq!(value, None);
        
        conn.disconnect().await.unwrap();
        assert!(!conn.is_connected().await);
    }

    #[tokio::test]
    async fn test_network_state() {
        let conn = VeilidConnection::new().await.unwrap();
        let state = conn.get_network_state().await.unwrap();
        // Depending on environment, we may or may not be connected yet, but the call should succeed.
        assert!(state.mode.contains("Veilid") || state.mode.contains("Fallback"));
    }
} 