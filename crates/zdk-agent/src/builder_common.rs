//! Common builder infrastructure for all agents

use zdk_core::{Error, Result};

/// Common builder fields for all agents
#[derive(Debug, Clone)]
pub struct AgentBuilderCore {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
}

impl AgentBuilderCore {
    /// Create a new builder core
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
        }
    }

    /// Set the name
    pub fn with_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Set the description
    pub fn with_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Validates and returns (name, description) or error
    ///
    /// # Arguments
    /// * `agent_type` - The type of agent for error messages (e.g., "LLMAgent")
    /// * `default_desc` - Default description if none provided
    pub fn validate(&self, agent_type: &str, default_desc: &str) -> Result<(String, String)> {
        let name = self
            .name
            .clone()
            .ok_or_else(|| Error::Config(format!("{} name is required", agent_type)))?;
        let description = self
            .description
            .clone()
            .unwrap_or_else(|| default_desc.to_string());
        Ok((name, description))
    }
}

impl Default for AgentBuilderCore {
    fn default() -> Self {
        Self::new()
    }
}
