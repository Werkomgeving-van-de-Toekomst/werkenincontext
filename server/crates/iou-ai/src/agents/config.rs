//! Agent configuration types.

use serde::{Deserialize, Serialize};

/// Global agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub research: ResearchAgentConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            research: ResearchAgentConfig::default(),
        }
    }
}

/// Research Agent specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchAgentConfig {
    /// Maximum number of similar documents to retrieve
    pub max_similar_documents: usize,

    /// Minimum similarity score threshold (0.0 - 1.0)
    pub similarity_threshold: f32,

    /// Minimum occurrence frequency for mandatory section (0.0 - 1.0)
    pub mandatory_threshold: f32,

    /// Whether to use AI provider for enhanced analysis
    pub use_ai_enhancement: bool,
}

impl Default for ResearchAgentConfig {
    fn default() -> Self {
        Self {
            max_similar_documents: 5,
            similarity_threshold: 0.7,
            mandatory_threshold: 0.9,
            use_ai_enhancement: true,
        }
    }
}

impl ResearchAgentConfig {
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.similarity_threshold) {
            return Err(format!(
                "similarity_threshold must be between 0.0 and 1.0, got {}",
                self.similarity_threshold
            ));
        }
        if !(0.0..=1.0).contains(&self.mandatory_threshold) {
            return Err(format!(
                "mandatory_threshold must be between 0.0 and 1.0, got {}",
                self.mandatory_threshold
            ));
        }
        if self.max_similar_documents == 0 {
            return Err("max_similar_documents must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_agent_config_default() {
        let config = ResearchAgentConfig::default();
        assert_eq!(config.max_similar_documents, 5);
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.mandatory_threshold, 0.9);
        assert!(config.use_ai_enhancement);
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.research.max_similar_documents, 5);
    }

    #[test]
    fn test_research_agent_config_validate_valid() {
        let config = ResearchAgentConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_research_agent_config_validate_invalid_similarity_threshold() {
        let config = ResearchAgentConfig {
            similarity_threshold: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_research_agent_config_validate_invalid_mandatory_threshold() {
        let config = ResearchAgentConfig {
            mandatory_threshold: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_research_agent_config_validate_zero_max_documents() {
        let config = ResearchAgentConfig {
            max_similar_documents: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
