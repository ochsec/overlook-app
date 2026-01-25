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
    /// Command to run (e.g., "uvx", "npx", "node")
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
    /// Environment variables to set
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Whether this server is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Minimum interval between tool calls in milliseconds (optional, for rate limiting)
    #[serde(default)]
    pub rate_limit_ms: Option<u64>,
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
            env: HashMap::new(),
            enabled: true,
            rate_limit_ms: None,
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

        for server_config in servers {
            if !server_config.enabled {
                continue;
            }

            match self.connect_server(server_config.clone()).await {
                Ok(tool_count) => {
                    results.push(Ok(format!(
                        "Connected to '{}': {} tools available",
                        server_config.name, tool_count
                    )));
                }
                Err(e) => {
                    results.push(Err(anyhow::anyhow!(
                        "Failed to connect to '{}': {}",
                        server_config.name,
                        e
                    )));
                }
            }
        }

        results
    }

    /// Connect to a single MCP server
    async fn connect_server(&self, config: McpServerConfig) -> Result<usize> {
        let cmd = Command::new(&config.command);

        // Configure the command with args and env
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

        // Connect to the server using serve_client
        let service = serve_client((), transport)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to MCP server: {:?}", e))?;

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
        let mut all_tools = Vec::new();

        for (server_name, server) in connected.iter() {
            for tool in &server.tools {
                all_tools.push((server_name.clone(), tool.clone()));
            }
        }

        all_tools
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<String> {
        let limiters = self.rate_limiters.lock().await;
        if let Some(limiter) = limiters.get(server_name) {
            let limiter = limiter.clone();
            drop(limiters);
            limiter.acquire().await;
        } else {
            drop(limiters);
        }

        let connected = self.connected.lock().await;

        let server = connected
            .get(server_name)
            .context(format!("Server '{}' is not connected", server_name))?;

        let result = server
            .service
            .call_tool(CallToolRequestParam {
                name: tool_name.to_string().into(),
                arguments,
            })
            .await
            .context("Failed to call tool")?;

        // Convert result to string
        let mut output = String::new();
        for content in result.content {
            if let Some(text) = content.as_text() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&text.text);
            }
        }

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

pub fn load_mcp_config() -> Result<McpConfig> {
    if let Some(app_dir) = dirs::config_dir().map(|d| d.join("overlook")) {
        let config_path = app_dir.join(MCP_CONFIG_FILE);

        if let Ok(config_data) = std::fs::read_to_string(&config_path) {
            let config: McpConfig =
                serde_json::from_str(&config_data).context("Failed to parse MCP config")?;
            return Ok(config);
        }
    }
    Ok(McpConfig::default())
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
