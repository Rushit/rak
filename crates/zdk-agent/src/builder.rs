use crate::llm_agent::LLMAgent;
use zdk_core::{Agent, Error, Result, Tool, Toolset, LLM};
use std::collections::HashMap;
use std::sync::Arc;

pub struct LLMAgentBuilder {
    name: Option<String>,
    description: Option<String>,
    model: Option<Arc<dyn LLM>>,
    system_instruction: Option<String>,
    sub_agents: Vec<Arc<dyn Agent>>,
    tools: HashMap<String, Arc<dyn Tool>>,
    toolsets: Vec<Arc<dyn Toolset>>,
}

impl LLMAgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            model: None,
            system_instruction: None,
            sub_agents: Vec::new(),
            tools: HashMap::new(),
            toolsets: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn model(mut self, model: Arc<dyn LLM>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn system_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.system_instruction = Some(instruction.into());
        self
    }

    pub fn sub_agent(mut self, agent: Arc<dyn Agent>) -> Self {
        self.sub_agents.push(agent);
        self
    }

    pub fn tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }

    pub fn tools(mut self, tools: Vec<Arc<dyn Tool>>) -> Self {
        for tool in tools {
            self.tools.insert(tool.name().to_string(), tool);
        }
        self
    }

    pub fn toolset(mut self, toolset: Arc<dyn Toolset>) -> Self {
        self.toolsets.push(toolset);
        self
    }

    pub fn build(self) -> Result<LLMAgent> {
        let name = self
            .name
            .ok_or_else(|| Error::Other(anyhow::anyhow!("Agent name is required")))?;
        let description = self
            .description
            .unwrap_or_else(|| "An LLM-powered agent".to_string());
        let model = self
            .model
            .ok_or_else(|| Error::Other(anyhow::anyhow!("Model is required")))?;

        Ok(LLMAgent {
            name,
            description,
            model,
            system_instruction: self.system_instruction,
            sub_agents: self.sub_agents,
            tools: self.tools,
            toolsets: self.toolsets,
        })
    }
}

impl Default for LLMAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
