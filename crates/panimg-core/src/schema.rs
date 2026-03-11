use serde::Serialize;

/// Describes a single parameter for a command (used by --schema).
#[derive(Debug, Clone, Serialize)]
pub struct ParamSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParamType,
    pub required: bool,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<ParamRange>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Integer,
    Float,
    Boolean,
    Path,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParamRange {
    pub min: f64,
    pub max: f64,
}

/// Full schema for a command.
#[derive(Debug, Clone, Serialize)]
pub struct CommandSchema {
    pub command: String,
    pub description: String,
    pub params: Vec<ParamSchema>,
}
