
use crate::types::{Provider, LLMRequest, LLMResponse, Message, Choice, UsageInfo, ToolCall};
use uuid::Uuid;

/// Normalizza i tools in formato OpenAI (`{"type":"function","function":{name,...}}`).
/// Accetta sia formato Anthropic (`{"name":..,"input_schema":..}`) che OpenAI
/// (`{"function":{"name":..,"parameters":..}}`) in ingresso (REVIEW 31/08:
/// prima il Nexus rompeva i tools OpenAI perché assumeva solo il formato Anthropic).
fn normalize_tools(tools: &serde_json::Value) -> Option<serde_json::Value> {
    let tools_arr = tools.as_array()?;
    let openai_tools: Vec<serde_json::Value> = tools_arr
        .iter()
        .filter_map(|t| {
            let (name, description, parameters) = if t.get("function").is_some() {
                // Formato OpenAI già pronto
                let f = &t["function"];
                (
                    f.get("name").cloned().unwrap_or(serde_json::Value::Null),
                    f.get("description").cloned().unwrap_or(serde_json::Value::Null),
                    f.get("parameters").cloned().unwrap_or(serde_json::Value::Null),
                )
            } else {
                // Formato Anthropic
                (
                    t.get("name").cloned().unwrap_or(serde_json::Value::Null),
                    t.get("description").cloned().unwrap_or(serde_json::Value::Null),
                    t.get("input_schema").cloned().unwrap_or(serde_json::Value::Null),
                )
            };
            if name.is_null() || !name.is_string() {
                return None;
            }
            Some(serde_json::json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": parameters,
                }
            }))
        })
        .collect();
    if openai_tools.is_empty() {
        None
    } else {
        Some(serde_json::json!(openai_tools))
    }
}

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

/// Invoca un provider OpenAI-compatibile in STREAMING (SSE) e ritorna lo stream di bytes.
/// Il body è identico a try_openai_compatible ma con `"stream": true`.
pub async fn stream_openai_compatible(
    client: &reqwest::Client,
    provider: &Provider,
    request: &LLMRequest,
) -> Result<impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>>, anyhow::Error> {
    let target_model = &provider.model;

    let mut system_prompt = String::new();
    let mut messages = Vec::new();
    for msg in &request.messages {
        if msg.role == "system" {
            if !system_prompt.is_empty() {
                system_prompt.push('\n');
            }
            system_prompt.push_str(&msg.content);
        } else if msg.role == "tool" && msg.tool_call_id.is_some() {
            // Gemini (via OpenAI-compat) richiede function_response.name nel tool result.
            let mut tool_msg = serde_json::json!({
                "role": "tool",
                "content": msg.content,
                "tool_call_id": msg.tool_call_id
            });
            if let Some(ref name) = msg.tool_name {
                if !name.is_empty() {
                    tool_msg["name"] = serde_json::json!(name);
                }
            }
            messages.push(tool_msg);
        } else if msg.role == "assistant" && msg.tool_calls_json.is_some() {
            let mut m = serde_json::json!({
                "role": "assistant",
                "content": msg.content
            });
            m["tool_calls"] = msg.tool_calls_json.clone().unwrap_or(serde_json::json!([]));
            messages.push(m);
        } else {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }
    }

    let mut body = serde_json::json!({
        "model": target_model,
        "messages": messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "temperature": request.temperature.unwrap_or(0.7),
        "stream": true,
    });

    if let Some(ref stop) = request.stop {
        if !stop.is_empty() {
            body["stop"] = serde_json::json!(stop);
        }
    }

    // System message nell'array messages (gemma locale incluso: il campo "system"
    // separato rompe il tool calling su gemma-4-E4B)
    if !system_prompt.is_empty() {
        let sys_msg = serde_json::json!({"role": "system", "content": system_prompt.trim()});
        if let Some(arr) = body["messages"].as_array_mut() {
            arr.insert(0, sys_msg);
        }
    }

    if let Some(ref tools) = request.tools {
        if let Some(tools_arr) = tools.as_array() {
            let openai_tools: Vec<serde_json::Value> = tools_arr.iter().map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t["name"],
                        "description": t["description"],
                        "parameters": t["input_schema"]
                    }
                })
            }).collect();
            body["tools"] = serde_json::json!(openai_tools);
        } else {
            body["tools"] = tools.clone();
        }
    }

    let base = provider.base_url.trim_end_matches('/');
    let url = if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{}/chat/completions", base)
    } else {
        format!("{}/v1/chat/completions", base)
    };

    let all_keys = resolve_provider_keys(provider);
    let selected_key = pick_ordered_keys(&all_keys).into_iter().next().unwrap_or_default();

    let mut req = client.post(&url)
        .timeout(std::time::Duration::from_secs(120))
        .json(&body);

    match provider.auth_type.as_str() {
        "bearer" => {
            if !selected_key.is_empty() {
                req = req.bearer_auth(&selected_key);
            }
        }
        "api-key" => {
            if !selected_key.is_empty() {
                req = req.header("x-api-key", &selected_key);
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

    Ok(resp.bytes_stream())
}

/// Contatore globale per il round-robin deterministico delle chiavi API.
/// `fetch_add` garantisce rotazione equa tra richieste concorrenti.
static KEY_ROUND_ROBIN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Risolve tutte le chiavi API configurate per il provider (da campo DB o env var dedicata).
pub fn resolve_provider_keys(provider: &Provider) -> Vec<String> {
    let env_name = format!("{}_API_KEY", provider.name.to_uppercase().replace('-', "_"));
    let raw = if let Some(ref k) = provider.api_key {
        if !k.trim().is_empty() {
            k.clone()
        } else {
            std::env::var(&env_name).unwrap_or_default()
        }
    } else {
        match provider.name.as_str() {
            "gemini-free-tier" | "gemini" => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
            "groq-free-pool" | "groq" => std::env::var("GROQ_API_KEY").unwrap_or_default(),
            "sambanova-cloud" | "sambanova" => std::env::var("SAMBANOVA_API_KEY").unwrap_or_default(),
            "cerebras-fast" | "cerebras" => std::env::var("CEREBRAS_API_KEY").unwrap_or_default(),
            "mistral-ai-cloud" | "mistral" => std::env::var("MISTRAL_API_KEY").unwrap_or_default(),
            "nvidia-nim-cloud" | "nvidia" => std::env::var("NVIDIA_API_KEY").unwrap_or_default(),
            "openrouter-free-pool" | "openrouter" => std::env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            _ => std::env::var(&env_name).unwrap_or_default(),
        }
    };

    if raw.is_empty() {
        return Vec::new();
    }

    raw.split(&[',', ';'][..])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Ritorna la lista di chiavi ordinata a partire dall'indice di round-robin corrente.
pub fn pick_ordered_keys(keys: &[String]) -> Vec<String> {
    if keys.is_empty() {
        return Vec::new();
    }
    let start_idx = KEY_ROUND_ROBIN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % keys.len();
    let mut ordered = Vec::with_capacity(keys.len());
    for i in 0..keys.len() {
        ordered.push(keys[(start_idx + i) % keys.len()].clone());
    }
    ordered
}

async fn try_openai_compatible(
    client: &reqwest::Client,
    provider: &Provider,
    request: &LLMRequest,
) -> Result<LLMResponse, anyhow::Error> {
    let target_model = &provider.model;

    // Costruisci i messaggi esplicitamente (non serializzare Message Rust):
    // serve il formato OpenAI corretto con `name` per i tool_result (Gemini).
    let mut system_prompt = String::new();
    let mut messages = Vec::new();
    for msg in &request.messages {
        if msg.role == "system" {
            if !system_prompt.is_empty() {
                system_prompt.push('\n');
            }
            system_prompt.push_str(&msg.content);
        } else if msg.role == "tool" && msg.tool_call_id.is_some() {
            let mut tool_msg = serde_json::json!({
                "role": "tool",
                "content": msg.content,
                "tool_call_id": msg.tool_call_id
            });
            if let Some(ref name) = msg.tool_name {
                if !name.is_empty() {
                    tool_msg["name"] = serde_json::json!(name);
                }
            }
            messages.push(tool_msg);
        } else if msg.role == "assistant" && msg.tool_calls_json.is_some() {
            let mut m = serde_json::json!({
                "role": "assistant",
                "content": msg.content
            });
            m["tool_calls"] = msg.tool_calls_json.clone().unwrap_or(serde_json::json!([]));
            messages.push(m);
        } else {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }
    }

    let mut body = serde_json::json!({
        "model": target_model,
        "messages": messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "temperature": request.temperature.unwrap_or(0.7),
    });

    if !system_prompt.is_empty() {
        let sys_msg = serde_json::json!({"role": "system", "content": system_prompt.trim()});
        if let Some(arr) = body["messages"].as_array_mut() {
            arr.insert(0, sys_msg);
        }
    }

    if let Some(ref tools) = request.tools {
        if let Some(openai_tools) = normalize_tools(tools) {
            body["tools"] = openai_tools;
        } else {
            body["tools"] = tools.clone();
        }
    }

    let base = provider.base_url.trim_end_matches('/');
    let url = if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{}/chat/completions", base)
    } else {
        format!("{}/v1/chat/completions", base)
    };

    let all_keys = resolve_provider_keys(provider);
    let keys_to_try = if all_keys.is_empty() {
        vec!["".to_string()]
    } else {
        pick_ordered_keys(&all_keys)
    };

    let mut last_err = anyhow::anyhow!("Nessuna chiave disponibile per {}", provider.name);

    for (k_idx, selected_key) in keys_to_try.iter().enumerate() {
        let mut req = client.post(&url)
            .timeout(std::time::Duration::from_secs(20))
            .json(&body);

        match provider.auth_type.as_str() {
            "bearer" => {
                if !selected_key.is_empty() {
                    req = req.bearer_auth(selected_key);
                }
            }
            "api-key" => {
                if !selected_key.is_empty() {
                    req = req.header("x-api-key", selected_key);
                }
            }
            _ => {}
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                last_err = e.into();
                continue;
            }
        };

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            // Se riceviamo 429 e abbiamo altre chiavi nel pool dello stesso provider, proviamo la successiva
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS && k_idx + 1 < keys_to_try.len() {
                tracing::warn!("⚠️ Chiave API #{} per '{}' ha ricevuto 429. Tentativo chiave successiva nel pool...", k_idx + 1, provider.name);
                last_err = anyhow::anyhow!("Provider {} HTTP {}: {}", provider.name, status, err_text);
                continue;
            }
            anyhow::bail!("Provider {} HTTP {}: {}", provider.name, status, err_text);
        }

        let json: serde_json::Value = resp.json().await?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
    let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

    // Estrai tool_calls (OpenAI) → ToolCall (interno)
    let mut tool_calls = None;
    if let Some(tc_array) = json["choices"][0]["message"]["tool_calls"].as_array() {
        let mut calls = Vec::new();
        for tc in tc_array {
            let id = tc["id"].as_str().unwrap_or("").to_string();
            let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
            let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
            let input: serde_json::Value = serde_json::from_str(args_str)
                .unwrap_or(serde_json::json!({}));
            calls.push(ToolCall {
                id: if id.is_empty() { format!("toolu_{}", Uuid::new_v4()) } else { id },
                call_type: "tool_use".to_string(),
                name,
                input,
            });
        }
        if !calls.is_empty() {
            tool_calls = Some(calls);
        }
    }

    let finish_reason = json["choices"][0]["finish_reason"]
        .as_str()
        .unwrap_or("stop")
        .to_string();

        return Ok(LLMResponse {
            id: format!("chatcmpl-{}", Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: target_model.to_string(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    tool_call_id: None,
                    tool_name: None,
                    tool_calls_json: None,
                    role: "assistant".to_string(),
                    content,
                },
                finish_reason,
            }],
            usage: UsageInfo {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
                estimated_cost_usd: 0.0, // Calcolato dal router in base al catalogo
            },
            provider_used: provider.name.clone(),
            tool_calls,
        });
    }

    Err(last_err)
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

    // Mappatura esplicita role/content: i campi tool_* della struct Message
    // (tool_call_id, tool_name, tool_calls_json) non sono accettati dall'API
    // native Gemini e anche con skip_serializing_if restano un rischio quando Some.
    let clean_messages: Vec<serde_json::Value> = request.messages.iter().map(|m| {
        serde_json::json!({
            "role": m.role,
            "content": m.content,
        })
    }).collect();

    let body = serde_json::json!({
        "model": request.model.as_deref().unwrap_or(&provider.model),
        "messages": clean_messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "temperature": request.temperature.unwrap_or(0.7),
    });

    let resp = client.post(&url)
        .timeout(std::time::Duration::from_secs(20))
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
                tool_call_id: None,
            tool_name: None,
            tool_calls_json: None,
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
        tool_calls: None,
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
                tool_call_id: None,
            tool_name: None,
            tool_calls_json: None,
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
        tool_calls: None,
    })
}
