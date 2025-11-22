//! Connection parameters for MCP servers

use std::collections::HashMap;

/// Parameters for connecting to an MCP server via stdio subprocess
#[derive(Debug, Clone)]
pub struct StdioConnectionParams {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

impl StdioConnectionParams {
    /// Create new connection parameters with the given command
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// Add a command-line argument
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add an environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

