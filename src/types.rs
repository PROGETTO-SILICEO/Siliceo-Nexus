use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: Option<i64>,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub auth_type: String, // bearer, api-key, anthropic, gemini-query, none
    pub model: String,
    pub priority: u32,
    pub tier: String, // free, paid, local
    pub tags: Vec<String>, // chitchat, coding, reasoning, vision, tool_supported
    pub tpm_limit: u32, // Token al minuto max
    pub rpm_limit: u32, // Richieste al minuto max
    pub enabled: bool,
    pub cooldown_until: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderInput {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    pub model: String,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default = "default_tier")]
    pub tier: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_tpm")]
    pub tpm_limit: u32,
    #[serde(default = "default_rpm")]
    pub rpm_limit: u32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_auth_type() -> String { "bearer".to_string() }
fn default_priority() -> u32 { 100 }
fn default_tier() -> String { "free".to_string() }
fn default_tpm() -> u32 { 32000 }
fn default_rpm() -> u32 { 15 }
fn default_enabled() -> bool { true }

#[derive(Debug, Clone, Deserialize)]
pub struct LLMRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub tools: Option<serde_json::Value>,
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Nome della funzione (per Gemini: function_response.name richiesto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool calls in formato OpenAI (assistant message): [{"id","type","function":{...}}]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls_json: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct LLMResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: UsageInfo,
    pub provider_used: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogItem {
    pub id: Option<i64>,
    pub provider_name: String,
    pub model_id: String,
    pub prompt_cost_per_1m: f64,
    pub completion_cost_per_1m: f64,
    pub context_length: u32,
    pub is_free: bool,
    pub capabilities: Vec<String>,
    pub last_updated: String,
}
