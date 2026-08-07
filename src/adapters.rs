
use crate::types::{Provider, LLMRequest, LLMResponse, Message, Choice, UsageInfo};
use uuid::Uuid;

pub async fn dispatch_request(
    client: &reqwest::Client,
    provider: &Provider,
    request: &LLMRequest,
) -> Result<LLMResponse, anyhow::Error> {
    match provider.auth_type.as_str() {
        "anthropic" => try_anthropic(client, provider, request).await,
        "gemini-query" => try_gemini_native(client, provider, request).await,
        _ => try_openai_compatible(client, provider, request).await,
    }
}

async fn try_openai_compatible(
    client: &reqwest::Client,
    provider: &Provider,
    request: &LLMRequest,
) -> Result<LLMResponse, anyhow::Error> {
    let target_model = request.model.as_deref().unwrap_or(&provider.model);
    let mut body = serde_json::json!({
        "model": target_model,
        "messages": request.messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "temperature": request.temperature.unwrap_or(0.7),
    });

    if let Some(ref tools) = request.tools {
        body["tools"] = tools.clone();
    }

    let url = if provider.base_url.contains("/v1/chat/completions") {
        provider.base_url.clone()
    } else {
        format!("{}/v1/chat/completions", provider.base_url.trim_end_matches('/'))
    };

    let mut req = client.post(&url)
        .timeout(std::time::Duration::from_secs(45))
        .json(&body);

    match provider.auth_type.as_str() {
        "bearer" => {
            if let Some(key) = &provider.api_key {
                if !key.is_empty() {
                    req = req.bearer_auth(key);
                }
            }
        }
        "api-key" => {
            if let Some(key) = &provider.api_key {
                if !key.is_empty() {
                    req = req.header("x-api-key", key.as_str());
                }
            }
        }
        _ => {}
    }

    let resp = req.send().await?;
    let status = resp.status();

    if !status.is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Provider {} HTTP {}: {}", provider.name, status, err_text);
    }

    let json: serde_json::Value = resp.json().await?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
    let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

    Ok(LLMResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: target_model.to_string(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: UsageInfo {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            estimated_cost_usd: 0.0, // Calcolato dal router in base al catalogo
        },
        provider_used: provider.name.clone(),
    })
}

async fn try_gemini_native(
    client: &reqwest::Client,
    provider: &Provider,
    request: &LLMRequest,
) -> Result<LLMResponse, anyhow::Error> {
    let api_key = provider.api_key.as_deref().unwrap_or_default();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions?key={}",
        api_key
    );

    let body = serde_json::json!({
        "model": request.model.as_deref().unwrap_or(&provider.model),
        "messages": request.messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "temperature": request.temperature.unwrap_or(0.7),
    });

    let resp = client.post(&url)
        .timeout(std::time::Duration::from_secs(45))
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Gemini HTTP {}: {}", status, err_text);
    }

    let json: serde_json::Value = resp.json().await?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
    let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

    Ok(LLMResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: provider.model.clone(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: UsageInfo {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            estimated_cost_usd: 0.0,
        },
        provider_used: provider.name.clone(),
    })
}

async fn try_anthropic(
    client: &reqwest::Client,
    provider: &Provider,
    request: &LLMRequest,
) -> Result<LLMResponse, anyhow::Error> {
    let mut system_prompt = String::new();
    let mut messages = Vec::new();

    for msg in &request.messages {
        if msg.role == "system" {
            system_prompt.push_str(&msg.content);
            system_prompt.push('\n');
        } else {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content,
            }));
        }
    }

    let mut body = serde_json::json!({
        "model": request.model.as_deref().unwrap_or(&provider.model),
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "messages": messages,
    });

    if !system_prompt.is_empty() {
        body["system"] = serde_json::json!(system_prompt.trim());
    }

    let url = if provider.base_url.contains("/v1/messages") {
        provider.base_url.clone()
    } else {
        format!("{}/v1/messages", provider.base_url.trim_end_matches('/'))
    };

    let mut req = client.post(&url).json(&body);

    if let Some(key) = &provider.api_key {
        req = req.header("x-api-key", key.as_str());
    }
    req = req.header("anthropic-version", "2023-06-01");

    let resp = req.send().await?;
    let status = resp.status();

    if !status.is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Anthropic HTTP {}: {}", status, err_text);
    }

    let json: serde_json::Value = resp.json().await?;

    let content = json["content"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|block| block["text"].as_str())
        .unwrap_or("")
        .to_string();

    let prompt_tokens = json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
    let completion_tokens = json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

    Ok(LLMResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: provider.model.clone(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: UsageInfo {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            estimated_cost_usd: 0.0,
        },
        provider_used: provider.name.clone(),
    })
}
