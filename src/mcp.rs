//! MCP (Model Context Protocol) client support.
//!
//! This module provides functionality to connect to MCP servers and use their tools.

use anyhow::{Context, Result};
use rmcp::{
    model::{CallToolRequestParam, Tool},
    service::{serve_client, RoleClient, RunningService},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};

/// Configuration for a single MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Unique name for this server
    pub name: String,
    /// Command to run (e.g., "uvx", "npx", "node") - for stdio transport
    #[serde(default)]
    pub command: String,
    /// Arguments to pass to the command - for stdio transport
    #[serde(default)]
    pub args: Vec<String>,
    /// URL for HTTP transport (if set, HTTP will be used instead of stdio)
    #[serde(default)]
    pub url: Option<String>,
    /// Environment variables to set
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Whether this server is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Minimum interval between tool calls in milliseconds (optional, for rate limiting)
    #[serde(default)]
    pub rate_limit_ms: Option<u64>,
    /// Optional list of tool names to expose (if set, only these tools will be available)
    #[serde(default)]
    pub tools_allowlist: Option<Vec<String>>,
    /// Optional list of tool names to block (if set, these tools will be hidden)
    #[serde(default)]
    pub tools_blocklist: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone)]
/// Rate limiter for tool calls
struct RateLimiter {
    min_interval: Duration,
    last_call: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            last_call: Arc::new(Mutex::new(Instant::now() - min_interval)),
        }
    }

    async fn acquire(&self) {
        let last_call = self.last_call.lock().await;
        let elapsed = last_call.elapsed();

        if elapsed < self.min_interval {
            let delay = self.min_interval - elapsed;
            drop(last_call);
            sleep(delay).await;
        }

        *self.last_call.lock().await = Instant::now();
    }
}

impl McpServerConfig {
    pub fn new(name: String, command: String, args: Vec<String>) -> Self {
        Self {
            name,
            command,
            args,
            url: None,
            env: HashMap::new(),
            enabled: true,
            rate_limit_ms: None,
            tools_allowlist: None,
            tools_blocklist: None,
        }
    }
}

/// All MCP server configurations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: Vec<McpServerConfig>,
}

/// A connected MCP server with its available tools
pub struct ConnectedServer {
    #[allow(dead_code)]
    pub config: McpServerConfig,
    pub service: RunningService<RoleClient, ()>,
    pub tools: Vec<Tool>,
}

/// Manager for all MCP connections
pub struct McpManager {
    config: StdMutex<McpConfig>,
    connected: Arc<Mutex<HashMap<String, ConnectedServer>>>,
    rate_limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new(McpConfig::default())
    }
}

impl McpManager {
    pub fn new(config: McpConfig) -> Self {
        Self {
            config: StdMutex::new(config),
            connected: Arc::new(Mutex::new(HashMap::new())),
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the current configuration
    #[allow(dead_code)]
    pub fn config(&self) -> McpConfig {
        self.config.lock().unwrap().clone()
    }

    /// Update the configuration
    pub fn set_config(&self, config: McpConfig) {
        *self.config.lock().unwrap() = config;
    }

    /// Set rate limit for a specific server (interval between calls in milliseconds)
    pub async fn set_rate_limit(&self, server_name: String, min_interval_ms: u64) {
        let mut limiters = self.rate_limiters.lock().await;
        limiters.insert(
            server_name,
            RateLimiter::new(Duration::from_millis(min_interval_ms)),
        );
    }

    /// Set default rate limit for all search-related servers (1000ms between calls)
    /// Also applies custom rate limits configured per server
    pub async fn set_default_search_rate_limit(&self) {
        let mut limiters = self.rate_limiters.lock().await;
        let config = self.config.lock().unwrap();

        for server in &config.servers {
            if let Some(interval_ms) = server.rate_limit_ms {
                limiters.insert(
                    server.name.clone(),
                    RateLimiter::new(Duration::from_millis(interval_ms)),
                );
            } else {
                let name = server.name.to_lowercase();
                if name.contains("search") || name.contains("brave") || name.contains("google") {
                    limiters.insert(
                        server.name.clone(),
                        RateLimiter::new(Duration::from_millis(1000)),
                    );
                }
            }
        }
    }

    /// Connect to all enabled MCP servers
    pub async fn connect_all(&self) -> Vec<Result<String>> {
        let mut results = Vec::new();
        let servers = self.config.lock().unwrap().servers.clone();

        println!("[MCP] Connecting to {} servers", servers.len());
        for server_config in servers {
            if !server_config.enabled {
                println!("[MCP] Skipping disabled server: {}", server_config.name);
                continue;
            }

            println!("[MCP] Connecting to server: {}", server_config.name);
            match self.connect_server(server_config.clone()).await {
                Ok(tool_count) => {
                    println!("[MCP] Connected to '{}': {} tools", server_config.name, tool_count);
                    results.push(Ok(format!(
                        "Connected to '{}': {} tools available",
                        server_config.name, tool_count
                    )));
                }
                Err(e) => {
                    eprintln!("[MCP] Failed to connect to '{}': {}", server_config.name, e);
                    results.push(Err(anyhow::anyhow!(
                        "Failed to connect to '{}': {}",
                        server_config.name,
                        e
                    )));
                }
            }
        }

        println!("[MCP] Connection complete, {} results", results.len());
        results
    }

    /// Connect to a single MCP server
    async fn connect_server(&self, config: McpServerConfig) -> Result<usize> {
        println!("[MCP] Connecting to server: {} (transport: {})", config.name, if config.url.is_some() { "HTTP" } else { "STDIO" });

        let service = if let Some(url) = &config.url {
            // HTTP transport
            let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(url.as_str());
            serve_client((), transport)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to MCP server via HTTP '{}': {:?}", config.name, e))?
        } else {
            // STDIO transport
            let cmd = Command::new(&config.command);
            let args = config.args.clone();
            let env = config.env.clone();

            let transport = TokioChildProcess::new(cmd.configure(move |c| {
                for arg in &args {
                    c.arg(arg);
                }
                for (key, value) in &env {
                    c.env(key, value);
                }
            }))
            .context("Failed to create child process transport")?;

            serve_client((), transport)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to MCP server via STDIO '{}': {:?}", config.name, e))?
        };

        // Get available tools
        let tools_response = service
            .list_tools(Default::default())
            .await
            .context("Failed to list tools")?;

        let tools = tools_response.tools;
        let tool_count = tools.len();

        // Store the connection
        let mut connected = self.connected.lock().await;
        connected.insert(
            config.name.clone(),
            ConnectedServer {
                config,
                service,
                tools,
            },
        );

        Ok(tool_count)
    }

    /// Disconnect from a server
    #[allow(dead_code)]
    pub async fn disconnect_server(&self, name: &str) -> Result<()> {
        let mut connected = self.connected.lock().await;
        if let Some(server) = connected.remove(name) {
            let _ = server.service.cancel().await;
        }
        Ok(())
    }

    /// Disconnect from all servers
    pub async fn disconnect_all(&self) -> Result<()> {
        let mut connected = self.connected.lock().await;
        for (_, server) in connected.drain() {
            let _ = server.service.cancel().await;
        }
        Ok(())
    }

    /// Get all available tools from all connected servers
    pub async fn list_all_tools(&self) -> Vec<(String, Tool)> {
        let connected = self.connected.lock().await;
        let config = self.config.lock().unwrap();
        let mut all_tools = Vec::new();

        println!("[MCP] Listing tools from {} connected servers", connected.len());
        for (server_name, server) in connected.iter() {
            // Get the server config to check allowlist/blocklist
            let server_config = config.servers.iter().find(|s| &s.name == server_name);

            for tool in &server.tools {
                // Check if tool should be exposed based on allowlist/blocklist
                let should_include = if let Some(cfg) = server_config {
                    if let Some(allowlist) = &cfg.tools_allowlist {
                        allowlist.contains(&tool.name.to_string())
                    } else if let Some(blocklist) = &cfg.tools_blocklist {
                        !blocklist.contains(&tool.name.to_string())
                    } else {
                        true
                    }
                } else {
                    true
                };

                if should_include {
                    println!("[MCP]   - {} (included)", tool.name);
                    all_tools.push((server_name.clone(), tool.clone()));
                } else {
                    println!("[MCP]   - {} (filtered out)", tool.name);
                }
            }
        }

        println!("[MCP] Total tools available: {}", all_tools.len());
        all_tools
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<String> {
        // Skip rate limiting here - it uses tokio::time::sleep which can hang
        // when called from run_on_tokio_async. Rate limiting is handled by the
        // connection setup instead.

        let connected = self.connected.lock().await;

        let server = connected
            .get(server_name)
            .context(format!("Server '{}' is not connected", server_name))?;

        println!("[MCP] call_tool: Calling tool '{}' on server '{}'", tool_name, server_name);
        let result = server
            .service
            .call_tool(CallToolRequestParam {
                name: tool_name.to_string().into(),
                arguments,
            })
            .await
            .context("Failed to call tool")?;
        println!("[MCP] call_tool: Tool call completed, {} content items", result.content.len());
        for (i, content) in result.content.iter().enumerate() {
            if let Some(text) = content.as_text() {
                println!("[MCP] call_tool: Content[{}]: {} chars", i, text.text.len());
            } else {
                println!("[MCP] call_tool: Content[{}]: (not text)", i);
            }
        }

        let mut output = String::new();
        for content in result.content {
            if let Some(text) = content.as_text() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&text.text);
            }
        }

        println!("[MCP] call_tool: Returning {} chars: {}", output.len(), &output[..output.len().min(500)]);
        Ok(output)
    }

    /// Check if any servers are connected
    #[allow(dead_code)]
    pub async fn has_connections(&self) -> bool {
        let connected = self.connected.lock().await;
        !connected.is_empty()
    }

    /// Get list of connected server names
    #[allow(dead_code)]
    pub async fn connected_servers(&self) -> Vec<String> {
        let connected = self.connected.lock().await;
        connected.keys().cloned().collect()
    }
}

// Storage functions for MCP config
const MCP_CONFIG_FILE: &str = "mcp_servers.json";

/// Returns the default You.com search MCP server config (HTTP transport, free tier)
pub fn default_youcom_server() -> McpServerConfig {
    McpServerConfig {
        name: "You.com Search".to_string(),
        command: String::new(),
        args: vec![],
        url: Some("https://api.you.com/mcp?profile=free".to_string()),
        env: HashMap::new(),
        enabled: true,
        rate_limit_ms: Some(1000),
        tools_allowlist: Some(vec!["you-search".to_string()]),
        tools_blocklist: None,
    }
}

pub fn load_mcp_config() -> Result<McpConfig> {
    let mut config = if let Some(app_dir) = dirs::config_dir().map(|d| d.join("overlook")) {
        let config_path = app_dir.join(MCP_CONFIG_FILE);

        if let Ok(config_data) = std::fs::read_to_string(&config_path) {
            println!("[MCP Config] Loading from: {:?}", config_path);
            if let Ok(config) = serde_json::from_str::<McpConfig>(&config_data) {
                println!("[MCP Config] Loaded {} servers", config.servers.len());
                for s in &config.servers {
                    println!("[MCP Config]   - {} (enabled={})", s.name, s.enabled);
                }
                config
            } else {
                println!("[MCP Config] Failed to parse config, using defaults");
                McpConfig::default()
            }
        } else {
            println!("[MCP Config] No config file found, using defaults");
            McpConfig::default()
        }
    } else {
        println!("[MCP Config] Cannot determine config dir, using defaults");
        McpConfig::default()
    };

    // Always ensure You.com search is available as a default server
    let has_youcom = config.servers.iter().any(|s| s.name.contains("You.com"));
    if !has_youcom {
        println!("[MCP Config] Adding default You.com server");
        config.servers.push(default_youcom_server());
    }

    println!("[MCP Config] Final config: {} servers", config.servers.len());
    Ok(config)
}

pub fn save_mcp_config(config: &McpConfig) -> Result<()> {
    let app_dir = dirs::config_dir()
        .map(|d| d.join("overlook"))
        .context("Cannot determine config directory")?;

    std::fs::create_dir_all(&app_dir)?;

    let config_json = serde_json::to_string_pretty(config)?;
    std::fs::write(app_dir.join(MCP_CONFIG_FILE), config_json)?;

    Ok(())
}