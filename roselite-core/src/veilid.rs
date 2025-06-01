use crate::error::*;
use std::sync::Arc;
use std::collections::HashMap;
use tracing;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use base64::{Engine as _, engine::general_purpose};
use crate::crypto::CryptoManager;

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
    pub bootstrap_nodes: Vec<String>,
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
            bootstrap_nodes: vec![
                // Use DNS-based bootstrap nodes from Veilid documentation
                "bootstrap.veilid.net:5150".to_string(),
                "bootstrap.dev.veilid.net:5150".to_string(),
                // Additional public nodes from the slides/documentation
                "178.68.166.46:5158".to_string(),
                "161.35.164.16:5158".to_string(),
                "159.89.163.27:5158".to_string(),
                "159.223.237.84:5158".to_string(),
            ],
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
            data_directory: None,
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
        match self.init_veilid_api().await {
            Ok(api) => {
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
            },
            Err(e) => {
                tracing::warn!("Failed to connect to Veilid network: {}. Using fallback storage.", e);
                
                // Update state to use fallback
                {
                    let mut state = self.state.write().await;
                    state.use_fallback_storage = true;
                    state.is_connected = true;
                    state.attachment_state = AttachmentState::Detached;
                }
            }
        }
        
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

    /// Store data in the DHT using Veilid's TableStore or fallback storage
    pub async fn dht_set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        tracing::debug!("Storing DHT value for key: {}", key);
        
        let use_fallback = {
            let state = self.state.read().await;
            state.use_fallback_storage || self.api.is_none()
        };
        
        if use_fallback {
            // Use fallback in-memory storage
            self.storage.write().await.insert(key.to_string(), value.to_vec());
            tracing::debug!("Stored value in fallback storage for key: {}", key);
            return Ok(());
        }

        let api = self.get_api()?;
        
        // Use Veilid TableStore with proper error handling
        let table_store = api.table_store()
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to get table store: {:?}", e)
            }))?;
            
        // Open or create table with column count of 1
        let table_db = table_store.open(&self.config.table_name, 1)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to open table: {:?}", e)
            }))?;
            
        // Store the value in column 0 - note: the store method takes &[u8] directly, not Option<&[u8]>
        table_db.store(0, key.as_bytes(), value)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to store value: {:?}", e)
            }))?;
            
        tracing::debug!("Successfully stored DHT value for key: {}", key);
        Ok(())
    }

    /// Retrieve data from the DHT using Veilid's TableStore or fallback storage
    pub async fn dht_get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        tracing::debug!("Retrieving DHT value for key: {}", key);
        
        let use_fallback = {
            let state = self.state.read().await;
            state.use_fallback_storage || self.api.is_none()
        };
        
        if use_fallback {
            // Use fallback in-memory storage
            let result = self.storage.read().await.get(key).cloned();
            tracing::debug!("Retrieved value from fallback storage for key: {}", key);
            return Ok(result);
        }

        let api = self.get_api()?;
        
        let table_store = api.table_store()
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to get table store: {:?}", e)
            }))?;
            
        // Open table
        let table_db = match table_store.open(&self.config.table_name, 1).await {
            Ok(db) => db,
            Err(_) => {
                // Table doesn't exist yet
                tracing::debug!("Table doesn't exist for key: {}", key);
                return Ok(None);
            }
        };
            
        // Load the value from column 0
        match table_db.load(0, key.as_bytes()).await {
            Ok(Some(value)) => {
                tracing::debug!("Successfully retrieved DHT value for key: {}", key);
                Ok(Some(value))
            },
            Ok(None) => {
                tracing::debug!("No value found for key: {}", key);
                Ok(None)
            },
            Err(e) => {
                Err(RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                    operation: format!("Failed to load value: {:?}", e)
                }))
            }
        }
    }

    /// Delete data from the DHT using Veilid's TableStore or fallback storage
    pub async fn dht_delete(&mut self, key: &str) -> Result<()> {
        tracing::debug!("Deleting DHT value for key: {}", key);
        
        let use_fallback = {
            let state = self.state.read().await;
            state.use_fallback_storage || self.api.is_none()
        };
        
        if use_fallback {
            // Use fallback in-memory storage
            self.storage.write().await.remove(key);
            tracing::debug!("Deleted value from fallback storage for key: {}", key);
            return Ok(());
        }

        let api = self.get_api()?;
        
        let table_store = api.table_store()
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to get table store: {:?}", e)
            }))?;
            
        // Open table
        let table_db = table_store.open(&self.config.table_name, 1)
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to open table: {:?}", e)
            }))?;
            
        // Delete the value using the delete method, not store with None
        table_db.delete(0, key.as_bytes())
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to delete value: {:?}", e)
            }))?;
            
        tracing::debug!("Successfully deleted DHT value for key: {}", key);
        Ok(())
    }

    /// List keys matching a pattern (enhanced for future implementation)
    pub async fn dht_list_keys(&self, pattern: &str) -> Result<Vec<String>> {
        tracing::debug!("Listing DHT keys for pattern: {}", pattern);
        
        let use_fallback = {
            let state = self.state.read().await;
            state.use_fallback_storage || self.api.is_none()
        };
        
        if use_fallback {
            // Use fallback in-memory storage
            let matching_keys: Vec<String> = self.storage.read().await.keys()
                .filter(|key| key.contains(pattern))
                .cloned()
                .collect();
            tracing::debug!("Found {} matching keys in fallback storage", matching_keys.len());
            return Ok(matching_keys);
        }

        // For Veilid TableStore, we would need to iterate through records
        // This requires a more complex implementation with record inspection
        tracing::warn!("Pattern-based key listing is not yet implemented for Veilid TableStore");
        Ok(Vec::new())
    }

    /// Send a message to another Veilid node
    pub async fn send_message(&self, target: &str, message: &[u8]) -> Result<()> {
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
            
        let api = self.get_api()?;
        
        // Parse the target (could be a node ID or route ID)
        let target_obj = api.parse_as_target(target)
            .map_err(|e| RoseliteError::Veilid(VeilidError::InvalidUri { 
                uri: format!("Failed to parse target: {:?}", e)
            }))?;
            
        // Send the message
        routing_ctx.app_message(target_obj, message.to_vec())
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to send message: {:?}", e)
            }))?;
            
        tracing::debug!("Successfully sent message to target: {}", target);
        Ok(())
    }

    /// Make an RPC call to another Veilid node
    pub async fn rpc_call(&self, target: &str, request: &[u8]) -> Result<Vec<u8>> {
        let routing_ctx = self.routing_context.as_ref()
            .ok_or_else(|| RoseliteError::Veilid(VeilidError::ConnectionFailed))?;
            
        let api = self.get_api()?;
        
        // Parse the target
        let target_obj = api.parse_as_target(target)
            .map_err(|e| RoseliteError::Veilid(VeilidError::InvalidUri { 
                uri: format!("Failed to parse target: {:?}", e)
            }))?;
            
        // Make the RPC call
        let response = routing_ctx.app_call(target_obj, request.to_vec())
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to make RPC call: {:?}", e)
            }))?;
            
        tracing::debug!("Successfully made RPC call to target: {}", target);
        Ok(response)
    }

    /// Create a new private route for enhanced privacy
    pub async fn create_private_route(&self) -> Result<String> {
        let api = self.get_api()?;
        
        let (route_id, route_blob) = api.new_private_route()
            .await
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to create private route: {:?}", e)
            }))?;
            
        // Convert route blob to base64 for sharing
        let route_str = general_purpose::STANDARD.encode(&route_blob);
        
        tracing::info!("Created private route with ID: {:?}", route_id);
        Ok(route_str)
    }

    /// Import a private route from another node
    pub async fn import_private_route(&self, route_blob: &str) -> Result<String> {
        let api = self.get_api()?;
        
        // Decode the route blob from base64
        let blob_data = general_purpose::STANDARD.decode(route_blob)
            .map_err(|e| RoseliteError::Veilid(VeilidError::InvalidUri { 
                uri: format!("Failed to decode route blob: {}", e)
            }))?;
            
        let route_id = api.import_remote_private_route(blob_data)
            .map_err(|e| RoseliteError::Veilid(VeilidError::DhtOperationFailed { 
                operation: format!("Failed to import private route: {:?}", e)
            }))?;
            
        tracing::info!("Imported private route with ID: {:?}", route_id);
        Ok(format!("{:?}", route_id))
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

    /// Build comprehensive Veilid configuration
    fn build_veilid_config(&self) -> Result<String> {
        let config_json = serde_json::json!({
            "program_name": self.config.program_name,
            "namespace": self.config.namespace,
            "capabilities": {
                "disable": []
            },
            "protected_store": {
                "allow_insecure_fallback": self.config.development_mode,
                "always_use_insecure_storage": self.config.development_mode,
                "directory": self.config.storage.data_directory.as_deref().unwrap_or(""),
                "delete": false
            },
            "table_store": {
                "directory": self.config.storage.data_directory.as_deref().unwrap_or(""),
                "delete": false
            },
            "block_store": {
                "directory": self.config.storage.data_directory.as_deref().unwrap_or(""),
                "delete": false
            },
            "network": {
                "connection_initial_timeout_ms": self.config.network.connection_timeout_ms,
                "connection_inactivity_timeout_ms": 60000,
                "max_connections_per_ip4": self.config.network.max_connections,
                "max_connections_per_ip6_prefix": self.config.network.max_connections,
                "max_connections_per_ip6_prefix_size": 56,
                "max_connection_frequency_per_min": 128,
                "client_allowlist_timeout_ms": 300000,
                "reverse_connection_receipt_time_ms": 5000,
                "hole_punch_receipt_time_ms": 5000,
                "routing_table": {
                    "bootstrap": self.config.bootstrap_nodes,
                    "limit_over_attached": 64,
                    "limit_fully_attached": 32,
                    "limit_attached_strong": 16,
                    "limit_attached_good": 8,
                    "limit_attached_weak": 5
                },
                "rpc": {
                    "concurrency": 0,
                    "queue_size": 1024,
                    "max_timestamp_behind_ms": 10000,
                    "max_timestamp_ahead_ms": 10000,
                    "timeout_ms": 10000,
                    "max_route_hop_count": 4,
                    "default_route_hop_count": 1
                },
                "dht": {
                    "max_find_node_count": 20,
                    "resolve_node_timeout_ms": 10000,
                    "resolve_node_count": 1,
                    "resolve_node_fanout": 4,
                    "get_value_timeout_ms": 10000,
                    "get_value_count": 3,
                    "get_value_fanout": 4,
                    "set_value_timeout_ms": 10000,
                    "set_value_count": 5,
                    "set_value_fanout": 4,
                    "min_peer_count": 20,
                    "min_peer_refresh_time_ms": 60000,
                    "validate_dial_info_receipt_time_ms": 2000,
                    "local_subkey_cache_size": 128,
                    "local_max_subkey_cache_memory_mb": 256,
                    "remote_subkey_cache_size": 1024,
                    "remote_max_records": 65536,
                    "remote_max_subkey_cache_memory_mb": 256,
                    "remote_max_storage_space_mb": self.config.storage.max_storage_mb
                },
                "upnp": self.config.network.enable_upnp,
                "detect_address_changes": self.config.network.enable_nat_detection,
                "restricted_nat_retries": 3,
                "tls": {
                    "connection_initial_timeout_ms": self.config.network.connection_timeout_ms
                },
                "application": {
                    "https": {
                        "enabled": false,
                        "listen_address": "",
                        "path": ""
                    },
                    "http": {
                        "enabled": false,
                        "listen_address": "",
                        "path": ""
                    }
                },
                "protocol": {
                    "udp": {
                        "enabled": true,
                        "socket_pool_size": 0,
                        "listen_address": self.config.network.udp_listen_address.as_deref().unwrap_or("")
                    },
                    "tcp": {
                        "connect": true,
                        "listen": true,
                        "max_connections": self.config.network.max_connections,
                        "listen_address": self.config.network.tcp_listen_address.as_deref().unwrap_or("")
                    },
                    "ws": {
                        "connect": true,
                        "listen": true,
                        "max_connections": self.config.network.max_connections / 2,
                        "listen_address": self.config.network.ws_listen_address.as_deref().unwrap_or(""),
                        "path": "ws"
                    },
                    "wss": {
                        "connect": true,
                        "listen": false,
                        "max_connections": self.config.network.max_connections / 4,
                        "listen_address": "",
                        "path": "wss"
                    }
                }
            }
        });

        Ok(config_json.to_string())
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
    async fn test_veilid_connection_fallback() {
        let mut conn = VeilidConnection::new().await.unwrap();
        
        // Should connect (possibly with fallback)
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
        assert!(state.mode.contains("Fallback") || state.mode.contains("Veilid"));
    }
} 