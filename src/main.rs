use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use reqwest::{redirect::Policy, Url};
use std::collections::VecDeque;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error};

mod types;
mod db;
mod router;
mod adapters;
mod dashboard;
mod catalog;
mod metrics;
mod streaming;

use types::{Provider, ProviderInput, LLMRequest, LLMResponse};
use metrics::MetricsRegistry;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub providers: Arc<RwLock<Vec<Provider>>>,
    pub client: reqwest::Client,
    pub request_window: Arc<Mutex<VecDeque<std::time::Instant>>>,
    pub metrics: Arc<Mutex<MetricsRegistry>>,
    pub gpu: Arc<RwLock<Option<serde_json::Value>>>,
    /// Cooldown provider in-memory: name -> istante UTC fino a cui è in pausa.
    pub cooldowns: Arc<RwLock<std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "siliceo_nexus=info".into()),
        )
        .init();

    info!("💎 Siliceo-Nexus starting...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/nexus.db".to_string());

    // Inizializzazione DB SQLite in WAL Mode
    std::fs::create_dir_all("data")?;
    let db = db::init_db(&database_url).await?;

    // Carica provider in memoria
    let providers_list = db::load_all_providers(&db).await;
    info!("   Loaded {} providers into Siliceo-Nexus memory pool", providers_list.len());

    for p in &providers_list {
        info!("   [P{}] [{}] {} -> {} ({:?})", p.priority, p.tier, p.name, p.model, p.tags);
    }

    let state = AppState {
        db: db.clone(),
        providers: Arc::new(RwLock::new(providers_list)),
        // Redirects can turn a safe-looking URL into an internal request. Each
        // endpoint is validated before use, so do not follow redirects.
        client: reqwest::Client::builder()
            .redirect(Policy::none())
            .build()?,
        request_window: Arc::new(Mutex::new(VecDeque::new())),
        metrics: Arc::new(Mutex::new(MetricsRegistry::new())),
        gpu: Arc::new(RwLock::new(None)),
        cooldowns: Arc::new(RwLock::new(std::collections::HashMap::new())),
    };

    // Avvia il loop di aggiornamento catalogo modelli in background (OpenRouter 24h sync)
    catalog::spawn_catalog_sync_loop(state.client.clone(), state.db.clone());
    // Avvia il polling della GPU reale dal nodo beellama (RTX 2070)
    spawn_gpu_polling(state.clone());
    // Avvia l'assessment proattivo dei provider (marca senza-chiave in cooldown)
    spawn_provider_assessment(state.clone());

    let app = Router::new()
        // Dashboard SPA
        .route("/", get(handle_dashboard))
        // Main OpenAI-compatible LLM endpoint
        .route("/v1/chat/completions", post(handle_chat_completions))
        // Native Anthropic API compatibility for Claude Code CLI
        .route("/v1/messages", post(handle_anthropic_messages))
        .route("/messages", post(handle_anthropic_messages))
        .route("/v1/v1/messages", post(handle_anthropic_messages))
        .route("/v1/models", get(handle_v1_models))
        .route("/models", get(handle_v1_models))
        .route("/v1/v1/models", get(handle_v1_models))
        // Management API
        .route("/providers", get(list_providers).post(create_provider))
        .route("/providers/fetch_models", post(handle_fetch_models))
        .route("/providers/:id", delete(delete_provider))
        .route("/providers/:id/test", post(handle_test_provider))
        .route("/providers/:id/set_model", post(handle_set_provider_model))
        // Catalog API
        .route("/catalog", get(handle_get_catalog))
        .route("/catalog/sync", post(handle_sync_catalog))
        // Health & Live Telemetry
        .route("/health", get(handle_health))
        .route("/stats", get(handle_stats))
        // Health check dell'SDK Anthropic: Claude Code chiama HEAD/GET {base}/api/hello
        // all'avvio. Se risponde 404, il client fallisce con "errore API".
        .route("/api/hello", get(handle_api_hello).head(handle_api_hello))
        .route("/v1/api/hello", get(handle_api_hello).head(handle_api_hello))
        .route("/messages/count_tokens", post(handle_count_tokens))
        .route("/v1/messages/count_tokens", post(handle_count_tokens))
        .with_state(state);

    let addr = std::env::var("NEXUS_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_string());
    ensure_network_auth_is_configured(&addr)?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("🌐 Siliceo-Nexus listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_dashboard() -> impl IntoResponse {
    (
        [(axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
        dashboard::render_dashboard()
    )
}

/// Polling della GPU reale dal nodo beellama (RTX 2070), ogni 2 secondi.
/// L'endpoint /gpu del beellama-switcher espone utilization, VRAM, temperatura.
fn spawn_gpu_polling(state: AppState) {
    let gpu_url = std::env::var("BELLAMA_GPU_URL")
        .unwrap_or_else(|_| "http://100.98.20.76:8080/gpu".to_string());
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        interval.tick().await; // primo tick immediato
        loop {
            interval.tick().await;
            let url = gpu_url.clone();
            match state.client.get(&url).timeout(std::time::Duration::from_secs(3)).send().await {
                Ok(resp) => {
                    let json = resp.json::<serde_json::Value>().await.ok();
                    if let Some(v) = json {
                        let mut gpu = state.gpu.write().await;
                        *gpu = Some(v);
                    }
                }
                Err(e) => {
                    let mut gpu = state.gpu.write().await;
                    *gpu = Some(serde_json::json!({
                        "status": "error",
                        "error": e.to_string()
                    }));
                }
            }
        }
    });
}

/// Assessment proattivo dei provider (ogni 5 minuti).
/// Marca in cooldown i provider senza chiave configurata (inutile provarli)
/// e azzera i cooldown scaduti per i provider con chiave. Tassonomia:
/// - chiave assente/vuota → cooldown 10 min (probabilmente non configurato)
/// - chiave presente → nessuna azione (gli errori reali sono gestiti in dispatch)
fn spawn_provider_assessment(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        interval.tick().await;
        loop {
            interval.tick().await;
            let now = chrono::Utc::now();
            let providers_snapshot = state.providers.read().await.clone();

            // Providers senza chiave e con auth_type che la richiede → cooldown 10 min
            let mut to_cooldown: Vec<String> = Vec::new();
            for p in &providers_snapshot {
                if !p.enabled { continue; }
                let needs_key = p.auth_type == "bearer" || p.auth_type == "api-key";
                if needs_key {
                    let has_key = p.api_key.as_ref().map_or(false, |k| !k.trim().is_empty());
                    if !has_key {
                        to_cooldown.push(p.name.clone());
                    }
                }
            }

            if !to_cooldown.is_empty() {
                let mut map = state.cooldowns.write().await;
                for name in &to_cooldown {
                    map.insert(name.clone(), now + chrono::Duration::seconds(600));
                }
                info!("🔍 [assessment] {} provider senza chiave messi in cooldown (10m)", to_cooldown.len());
            }
        }
    });
}

/// Applica un cooldown a un provider dopo un errore.
/// 429 → 120s, 401/403 → 300s (chiave non valida), 5xx → 60s, altri → 15s.
/// Backoff esponenziale: se già in cooldown, raddoppia (max 600s).
pub async fn apply_provider_cooldown(state: &AppState, provider: &Provider, status: Option<u16>) {
    let now = chrono::Utc::now();
    let base_secs = match status {
        Some(401) | Some(403) => 300,
        Some(429) => 120,
        Some(code) if code >= 500 && code < 600 => 60,
        _ => 15,
    };

    let duration = {
        let mut map = state.cooldowns.write().await;
        let multiplier = match map.get(&provider.name) {
            Some(until) if *until > now => 2,
            _ => 1,
        };
        let secs = (base_secs * multiplier).min(600);
        let until = now + chrono::Duration::seconds(secs as i64);
        map.insert(provider.name.clone(), until);
        secs
    };

    info!("⏸️ Cooldown '{}' ({}s) dopo errore {:?}", provider.name, duration, status);

    // Persistenza DB (fire-and-forget)
    let db = state.db.clone();
    let name = provider.name.clone();
    let until = now + chrono::Duration::seconds(duration as i64);
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE providers SET cooldown_until = ?, updated_at = datetime('now') WHERE name = ?")
            .bind(until.to_rfc3339())
            .bind(name)
            .execute(&db)
            .await;
    });
}

/// Azzera il cooldown di un provider dopo un successo.
pub async fn clear_provider_cooldown(state: &AppState, provider_name: &str) {
    let removed = state.cooldowns.write().await.remove(provider_name).is_some();
    if removed {
        let db = state.db.clone();
        let name = provider_name.to_string();
        tokio::spawn(async move {
            let _ = sqlx::query("UPDATE providers SET cooldown_until = NULL WHERE name = ?")
                .bind(name)
                .execute(&db)
                .await;
        });
    }
}

/// Estrae lo status HTTP da un messaggio di errore di dispatch_request.
/// Gli errori hanno forma "Provider X HTTP 429: ...".
fn error_status_code(err_str: &str) -> Option<u16> {
    for marker in ["HTTP ", "http ", "status ", "Status "] {
        if let Some(pos) = err_str.find(marker) {
            let after = &err_str[pos + marker.len()..];
            let num: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(code) = num.parse::<u16>() {
                return Some(code);
            }
        }
    }
    None
}

async fn handle_stats(State(state): State<AppState>) -> impl IntoResponse {    let providers_count = state.providers.read().await.len();

    let mem_info = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_total = 100.0;
    let mut mem_free = 50.0;
    for line in mem_info.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(v) = line.split_whitespace().nth(1) {
                mem_total = v.parse::<f64>().unwrap_or(100.0);
            }
        }
        if line.starts_with("MemAvailable:") {
            if let Some(v) = line.split_whitespace().nth(1) {
                mem_free = v.parse::<f64>().unwrap_or(50.0);
            }
        }
    }
    let memory_used_pct = (((mem_total - mem_free) / mem_total) * 100.0).round();

    let metrics = state.metrics.lock().await;
    let gpu = state.gpu.read().await.clone();
    let mut snapshot = metrics.snapshot();
    snapshot["providers_count"] = serde_json::json!(providers_count);
    snapshot["gpu"] = serde_json::json!(gpu);
    snapshot["system_memory_used_pct"] = serde_json::json!(memory_used_pct);
    snapshot["status"] = serde_json::json!("online");
    Json(snapshot)
}

async fn handle_health(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.metrics.lock().await.uptime_secs();
    let gpu = state.gpu.read().await.clone();
    let providers_count = state.providers.read().await.len();
    Json(serde_json::json!({
        "status": "ok",
        "service": "siliceo-nexus",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime,
        "providers_count": providers_count,
        "gpu": gpu,
    }))
}

/// Health check dell'SDK Anthropic. Claude Code lo chiama all'avvio (HEAD/GET).
/// Deve rispondere 200, altrimenti il client riporta un errore API.
async fn handle_api_hello() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "service": "siliceo-nexus",
        })),
    )
}

/// Claude Code in modalità interattiva chiama /v1/messages/count_tokens per stimare i costi.
/// Restituisce una stima basata sui caratteri del body.
async fn handle_count_tokens(
    headers: HeaderMap,
    request: axum::extract::Request,
) -> impl IntoResponse {
    verify_api_auth(&headers).ok();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let bytes = match axum::body::to_bytes(request.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return Json(serde_json::json!({"input_tokens": 0})),
    };
    let text = String::from_utf8_lossy(&bytes);
    let input_tokens = (text.chars().count() / 4).max(1) as u64;
    info!("🔢 [count_tokens] {} {} → stima {} token", method, path, input_tokens);
    Json(serde_json::json!({
        "input_tokens": input_tokens,
        "output_tokens": 0
    }))
}

async fn handle_chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<LLMRequest>,
) -> Result<Json<LLMResponse>, (StatusCode, String)> {
    verify_api_auth(&headers)?;
    enforce_inference_rate_limit(&state).await?;
    info!("📥 Incoming chat completions request (messages: {})", request.messages.len());

    // 1. Classificazione dell'intento in < 1ms
    let intent = router::classify_intent(&request);
    let requires_tools = request.tools.is_some();
    let intent_str = intent.as_str().to_string();
    let start = std::time::Instant::now();
    {
        let mut m = state.metrics.lock().await;
        m.record_start("chat", &intent_str);
    }

    // 2. Ottieni tutti i provider idonei ordinati per priorità per la cascata di failover
    let eligible = router::select_eligible_providers(&state.providers, Some(&state.cooldowns), intent, requires_tools).await;
    if eligible.is_empty() {
        state.metrics.lock().await.record_endpoint_error("chat");
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Nessun provider LLM disponibile".to_string()));
    }

    let mut last_error = String::new();

    // 3. Cascata di Failover Sequenziale
    for p in &eligible {
        info!("🚀 Tentativo con provider '{}' (model: {})", p.name, p.model);
        let attempt_start = std::time::Instant::now();
        match adapters::dispatch_request(&state.client, p, &request).await {
            Ok(response) => {
                let latency_ms = attempt_start.elapsed().as_millis() as u64;
                {
                    let mut m = state.metrics.lock().await;
                    m.record_provider(&p.name, true, latency_ms);
                }
                // Fallback dichiarativo: risposta vuota (content="" e nessun tool) = provider inutile
                let has_text = response.choices.first().map_or(false, |c| !c.message.content.trim().is_empty());
                let has_tools = response.tool_calls.as_ref().map_or(false, |t| !t.is_empty());
                if !has_text && !has_tools {
                    let err_str = format!("Provider {} ha risposto con contenuto vuoto", p.name);
                    apply_provider_cooldown(&state, p, Some(502)).await;
                    warn!("⚠️ {} — passo al candidato successivo", err_str);
                    last_error = err_str;
                    continue;
                }
                clear_provider_cooldown(&state, &p.name).await;
                info!("✅ Risposta consegnata da '{}' (model: {})", p.name, response.model);
                // Uso persistente per statistiche/costi
                let db = state.db.clone();
                let pname = p.name.clone();
                let model_used = response.model.clone();
                let pt = response.usage.prompt_tokens;
                let ct = response.usage.completion_tokens;
                let intent_c = intent_str.clone();
                tokio::spawn(async move {
                    let _ = db::insert_usage_log(&db, &pname, &model_used, pt, ct, 0.0, &intent_c).await;
                });
                return Ok(Json(response));
            }
            Err(e) => {
                let latency_ms = attempt_start.elapsed().as_millis() as u64;
                {
                    let mut m = state.metrics.lock().await;
                    m.record_provider(&p.name, false, latency_ms);
                }
                let err_str = e.to_string();
                apply_provider_cooldown(&state, p, error_status_code(&err_str)).await;
                warn!("⚠️ Provider '{}' fallito: {}. Passo al candidato successivo...", p.name, e);
                last_error = err_str;
            }
        }
    }

    state.metrics.lock().await.record_endpoint_error("chat");
    error!("❌ Tutti i provider della cascata sono falliti. Ultimo errore: {}", last_error);
    Err((StatusCode::BAD_GATEWAY, format!("Failover esausto. Ultimo errore: {}", last_error)))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AnthropicMessageRequest {
    pub model: Option<String>,
    pub messages: Vec<AnthropicMessage>,
    pub system: Option<serde_json::Value>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<serde_json::Value>,
}

async fn handle_v1_models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    verify_api_auth(&headers)?;
    enforce_inference_rate_limit(&state).await?;
    let rows = sqlx::query("SELECT model_id, provider_name FROM models_catalog LIMIT 2000")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut data = Vec::new();

    for row in rows {
        use sqlx::Row;
        let m_id: String = row.get("model_id");
        let prov: String = row.get("provider_name");
        data.push(serde_json::json!({
            "id": m_id,
            "object": "model",
            "created": 1700000000,
            "owned_by": prov
        }));
    }

    let claude_aliases = [
        "claude-sonnet-4-6",
        "claude-3-5-sonnet-20241022",
        "claude-3-7-sonnet-20250219",
        "claude-3-5-haiku-20241022",
        "claude-3-opus-20240229",
        "sonnet",
        "haiku",
        "opus"
    ];

    for alias in claude_aliases {
        data.push(serde_json::json!({
            "id": alias,
            "object": "model",
            "created": 1700000000,
            "owned_by": "anthropic"
        }));
    }

    Ok(Json(serde_json::json!({
        "object": "list",
        "data": data
    })))
}

/// Limita il contesto della richiesta: se i messaggi (escluso system) superano una
/// soglia stimata di token, tronca i più vecchi mantenendo gli ultimi N.
/// È una forma essenziale di context management per sessioni lunghe: senza questo,
/// una conversazione di 100+ messaggi esplode il contesto del provider.
fn trim_context(messages: Vec<types::Message>, max_est_tokens: usize) -> Vec<types::Message> {
    const CHARS_PER_TOKEN: usize = 4;

    // Separa system (va sempre preservato) dal resto
    let mut system_msgs = Vec::new();
    let mut history = Vec::new();
    for m in messages {
        if m.role == "system" {
            system_msgs.push(m);
        } else {
            history.push(m);
        }
    }

    // Stima token totali
    let total_est: usize = history.iter().map(|m| m.content.chars().count() / CHARS_PER_TOKEN).sum();
    if total_est <= max_est_tokens {
        return system_msgs.into_iter().chain(history).collect();
    }

    // Taglia dalla testa finché rientra (mantiene i più recenti)
    let mut trimmed = history;
    let mut est: usize = trimmed.iter().map(|m| m.content.chars().count() / CHARS_PER_TOKEN).sum();
    while est > max_est_tokens && trimmed.len() > 1 {
        let dropped = trimmed.remove(0);
        est = est.saturating_sub(dropped.content.chars().count() / CHARS_PER_TOKEN);
    }

    info!("✂️ [context] contesto troncato: {} messaggi mantenuti su ~{} token stimati", trimmed.len(), max_est_tokens);
    system_msgs.into_iter().chain(trimmed).collect()
}

async fn handle_anthropic_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(anthropic_req): Json<AnthropicMessageRequest>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    verify_api_auth(&headers)?;
    enforce_inference_rate_limit(&state).await?;
    info!("📥 Incoming Anthropic /v1/messages request (messages: {})", anthropic_req.messages.len());

    let mut llm_messages = Vec::new();

    if let Some(ref sys_val) = anthropic_req.system {
        let mut sys_str = String::new();
        if let Some(s) = sys_val.as_str() {
            sys_str = s.to_string();
        } else if let Some(arr) = sys_val.as_array() {
            for block in arr {
                if let Some(txt) = block.get("text").and_then(|t| t.as_str()) {
                    sys_str.push_str(txt);
                    sys_str.push('\n');
                }
            }
        } else {
            sys_str = sys_val.to_string();
        }

        if !sys_str.trim().is_empty() {
            llm_messages.push(types::Message {
                tool_call_id: None,
                tool_name: None,
                tool_calls_json: None,
                role: "system".to_string(),
                content: sys_str.trim().to_string(),
            });
        }
    }

    // Mappa tool_call_id → tool_name dai blocchi tool_use nella conversazione.
    // Necessaria per Gemini: il tool_result richiede function_response.name.
    let mut tool_names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for msg in &anthropic_req.messages {
        if let Some(arr) = msg.content.as_array() {
            for block in arr {
                if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                    let tu_id = block.get("id").and_then(|t| t.as_str()).unwrap_or("");
                    let tu_name = block.get("name").and_then(|t| t.as_str()).unwrap_or("");
                    if !tu_id.is_empty() && !tu_name.is_empty() {
                        tool_names.insert(tu_id.to_string(), tu_name.to_string());
                    }
                }
            }
        }
    }

    for msg in anthropic_req.messages {
        // Caso semplice: contenuto è una stringa
        if let Some(s) = msg.content.as_str() {
            llm_messages.push(types::Message {
                tool_call_id: None,
                tool_name: None,
                tool_calls_json: None,
                role: msg.role.clone(),
                content: s.to_string(),
            });
            continue;
        }

        // Contenuto è un array di content blocks
        if let Some(arr) = msg.content.as_array() {
            for block in arr {
                let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match block_type {
                    "text" => {
                        if let Some(txt) = block.get("text").and_then(|t| t.as_str()) {
                            llm_messages.push(types::Message {
                                tool_call_id: None,
                tool_name: None,
                tool_calls_json: None,
                                role: msg.role.clone(),
                                content: txt.to_string(),
                            });
                        }
                    }
                    // Tool use (dal modello): converti in assistant message con tool_calls (formato OpenAI)
                    "tool_use" => {
                        let tu_id = block.get("id").and_then(|t| t.as_str()).unwrap_or("");
                        let tu_name = block.get("name").and_then(|t| t.as_str()).unwrap_or("");
                        let tu_input = block.get("input").cloned().unwrap_or(serde_json::json!({}));
                        if !tu_id.is_empty() && !tu_name.is_empty() {
                            let tc = serde_json::json!({
                                "id": tu_id,
                                "type": "function",
                                "function": {
                                    "name": tu_name,
                                    "arguments": tu_input.to_string()
                                }
                            });
                            // Raggruppa: se l'ultimo messaggio è già assistant con tool_calls, appendi
                            if let Some(last) = llm_messages.last_mut() {
                                if last.role == "assistant" && last.tool_calls_json.is_some() {
                                    if let Some(arr) = last.tool_calls_json.as_mut().and_then(|v| v.as_array_mut()) {
                                        arr.push(tc);
                                        continue;
                                    }
                                }
                            }
                            llm_messages.push(types::Message {
                                tool_call_id: None,
                                tool_name: None,
                                role: "assistant".to_string(),
                                content: String::new(),
                                tool_calls_json: Some(serde_json::json!([tc])),
                            });
                        }
                    }
                    // Tool result da Claude Code → messaggio OpenAI role:"tool" con tool_call_id
                    "tool_result" => {
                        let tool_use_id = block.get("tool_use_id").and_then(|t| t.as_str()).unwrap_or("");
                        let content = extract_tool_result_content(&block["content"]);
                        let tool_name = if tool_use_id.is_empty() {
                            None
                        } else {
                            tool_names.get(tool_use_id).cloned()
                        };
                        info!("🔧 [anthropic] tool_result: id={} name={:?} mappa_size={}", tool_use_id, tool_name, tool_names.len());
                        llm_messages.push(types::Message {
                            tool_call_id: Some(tool_use_id.to_string()),
                            tool_name,
                            tool_calls_json: None,
                            role: "tool".to_string(),
                            content,
                        });
                    }
                    _ => {}
                }
            }
            continue;
        }

        // Fallback: content è qualcos'altro
        llm_messages.push(types::Message {
            tool_call_id: None,
            tool_name: None,
            tool_calls_json: None,
            role: msg.role.clone(),
            content: msg.content.to_string(),
        });
    }

    // Context management: limita la storia alle ultime ~24k token stimati
    // (preservando system) per non esplodere il contesto dei provider.
    let llm_messages = trim_context(llm_messages, 24_000);

    let llm_req = LLMRequest {
        messages: llm_messages,
        model: anthropic_req.model.clone(),
        max_tokens: anthropic_req.max_tokens,
        temperature: anthropic_req.temperature,
        stream: anthropic_req.stream,
        tools: anthropic_req.tools.clone(),
        stop: anthropic_req.stop_sequences.clone(),
    };

    let intent = router::classify_intent(&llm_req);
    let intent_str = intent.as_str().to_string();
    {
        let mut m = state.metrics.lock().await;
        m.record_start("anthropic", &intent_str);
    }
    // requires_tools: solo se la richiesta ha tools o se è un tool_result (role:"tool")
    let has_tools = llm_req.tools.as_ref().map_or(false, |t| t.as_array().map_or(false, |a| !a.is_empty()));
    let has_tool_result = llm_req.messages.iter().any(|m| m.role == "tool");
    info!("🔧 [anthropic] has_tools={} has_tool_result={} roles={:?}", has_tools, has_tool_result, llm_req.messages.iter().map(|m| m.role.as_str()).collect::<Vec<_>>());
    let eligible = router::select_eligible_providers(&state.providers, Some(&state.cooldowns), intent, has_tools || has_tool_result).await;

    if eligible.is_empty() {
        state.metrics.lock().await.record_endpoint_error("anthropic");
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Nessun provider LLM disponibile".to_string()));
    }

    // STREAMING: se il client chiede stream, rispondi in SSE (Anthropic format)
    info!("🔧 [anthropic] stream richiesto: {:?}", llm_req.stream);
    if llm_req.stream == Some(true) {
        return handle_anthropic_stream(&state, &eligible, &llm_req, &intent_str).await;
    }

    let mut last_err = String::new();
    for p in &eligible {
        info!("🚀 [anthropic] Tentativo con provider '{}' (model: {})", p.name, p.model);
        let attempt_start = std::time::Instant::now();
        match adapters::dispatch_request(&state.client, p, &llm_req).await {
            Ok(res) => {
                let latency_ms = attempt_start.elapsed().as_millis() as u64;
                {
                    let mut m = state.metrics.lock().await;
                    m.record_provider(&p.name, true, latency_ms);
                }
                let text_out = res.choices.first()
                    .map(|c| c.message.content.clone())
                    .unwrap_or_default();

                // Fallback dichiarativo: risposta vuota senza tool = provider inutile
                let has_tools_resp = res.tool_calls.as_ref().map_or(false, |t| !t.is_empty());
                if text_out.trim().is_empty() && !has_tools_resp {
                    let err_str = format!("Provider {} ha risposto con contenuto vuoto", p.name);
                    apply_provider_cooldown(&state, p, Some(502)).await;
                    warn!("⚠️ {} — passo al candidato successivo", err_str);
                    last_err = err_str;
                    continue;
                }
                clear_provider_cooldown(&state, &p.name).await;

                // Costruisci content array (text + tool_use blocks)
                let mut content_blocks: Vec<serde_json::Value> = Vec::new();
                if !text_out.is_empty() {
                    content_blocks.push(serde_json::json!({
                        "type": "text",
                        "text": text_out
                    }));
                }
                if let Some(ref tool_calls) = res.tool_calls {
                    for tc in tool_calls {
                        content_blocks.push(serde_json::json!({
                            "type": "tool_use",
                            "id": tc.id,
                            "name": tc.name,
                            "input": tc.input
                        }));
                    }
                }
                if content_blocks.is_empty() {
                    content_blocks.push(serde_json::json!({
                        "type": "text",
                        "text": ""
                    }));
                }

                let stop_reason = if res.tool_calls.as_ref().map_or(false, |t| !t.is_empty()) {
                    "tool_use"
                } else {
                    "end_turn"
                };

                let payload = serde_json::json!({
                    "id": format!("msg_{}", res.id),
                    "type": "message",
                    "role": "assistant",
                    "content": content_blocks,
                    "model": res.model,
                    "stop_reason": stop_reason,
                    "stop_sequence": null,
                    "usage": {
                        "input_tokens": res.usage.prompt_tokens,
                        "output_tokens": res.usage.completion_tokens
                    }
                });
                return axum::response::Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&payload).unwrap_or_default()))
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
            }
            Err(e) => {
                let latency_ms = attempt_start.elapsed().as_millis() as u64;
                {
                    let mut m = state.metrics.lock().await;
                    m.record_provider(&p.name, false, latency_ms);
                }
                let err_str = e.to_string();
                warn!("⚠️ [anthropic] Provider '{}' fallito: {}", p.name, err_str);
                apply_provider_cooldown(&state, p, error_status_code(&err_str)).await;
                last_err = err_str;
            }
        }
    }

    state.metrics.lock().await.record_endpoint_error("anthropic");
    Err((StatusCode::BAD_GATEWAY, format!("Failover esausto: {}", last_err)))
}

/// Estrae il contenuto testuale di un tool_result Anthropic.
/// Claude Code manda `content` come STRINGA o come ARRAY di blocchi
/// [{"type":"text","text":"..."}]. Gestiamo entrambi.
fn extract_tool_result_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(blocks) => {
            let parts: Vec<String> = blocks.iter().filter_map(|b| {
                b.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
            }).collect();
            if parts.is_empty() {
                content.to_string()
            } else {
                parts.join("\n")
            }
        }
        other => other.as_str().unwrap_or("").to_string(),
    }
}

/// Gestisce una richiesta Anthropic /v1/messages in STREAMING.
/// Prova i provider in cascata; al primo che accetta, converte il flusso SSE
/// OpenAI in eventi Anthropic e lo restituisce come `text/event-stream`.
async fn handle_anthropic_stream(
    state: &AppState,
    eligible: &[Provider],
    llm_req: &LLMRequest,
    intent_str: &str,
) -> Result<axum::response::Response, (StatusCode, String)> {
    use axum::body::Body;
    use axum::response::Response;

    let mut last_err = String::new();

    for p in eligible {
        info!("🚀 [anthropic:stream] Tentativo con provider '{}' (model: {})", p.name, p.model);

        // Solo provider OpenAI-compat supportano streaming in questo percorso
        if p.auth_type != "bearer" && p.auth_type != "api-key" && p.auth_type != "none" {
            continue;
        }

        let attempt_start = std::time::Instant::now();
        match adapters::stream_openai_compatible(&state.client, p, llm_req).await {
            Ok(stream) => {
                let latency_ms = attempt_start.elapsed().as_millis() as u64;
                {
                    let mut m = state.metrics.lock().await;
                    m.record_provider(&p.name, true, latency_ms);
                }
                clear_provider_cooldown(&state, &p.name).await;
                info!("✅ [anthropic:stream] Streaming avviato da '{}' ({}ms)", p.name, latency_ms);

                let message_id = format!("msg_{}", uuid::Uuid::new_v4().simple());
                let input_tokens = llm_req.messages.iter().map(|m| m.content.chars().count()).sum::<usize>() as u64 / 4;

                let anthropic_stream = streaming::AnthropicSseStream::new(
                    stream,
                    llm_req.model.clone().unwrap_or_default(),
                    message_id,
                    input_tokens,
                );

                let body = Body::from_stream(anthropic_stream);
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-cache")
                    .header("x-accel-buffering", "no")
                    .body(body)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
            }
            Err(e) => {
                let latency_ms = attempt_start.elapsed().as_millis() as u64;
                {
                    let mut m = state.metrics.lock().await;
                    m.record_provider(&p.name, false, latency_ms);
                }
                let err_str = e.to_string();
                apply_provider_cooldown(&state, p, error_status_code(&err_str)).await;
                warn!("⚠️ [anthropic:stream] Provider '{}' fallito: {}", p.name, e);
                last_err = err_str;
            }
        }
    }

    state.metrics.lock().await.record_endpoint_error("anthropic");
    Err((StatusCode::BAD_GATEWAY, format!("Streaming failover esausto: {}", last_err)))
}

pub fn redact_secrets(text: &str) -> String {
    let mut redacted = text.to_string();
    let key_prefixes = ["gsk_", "AIzaSy", "sk-ant-", "sk-proj-", "sk-or-", "nvapi-", "fw_", "csk-", "pplx-"];
    for prefix in key_prefixes {
        while let Some(start) = redacted.find(prefix) {
            let rest = &redacted[start..];
            let end_idx = rest.find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == '\\' || c == '}')
                .unwrap_or(rest.len());
            let raw_key = &rest[..end_idx];
            if raw_key.len() > 6 {
                let masked = format!("{}...{}", &raw_key[..4], &raw_key[raw_key.len()-2..]);
                redacted = redacted.replace(raw_key, &masked);
            } else {
                break;
            }
        }
    }
    redacted
}

fn configured_trusted_endpoint_hosts() -> Vec<String> {
    std::env::var("NEXUS_TRUSTED_ENDPOINT_HOSTS")
        .unwrap_or_default()
        .split(',')
        .map(|host| host.trim().trim_matches('[').trim_matches(']').to_ascii_lowercase())
        .filter(|host| !host.is_empty())
        .collect()
}

fn is_private_or_special_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || octets[0] == 0
                || octets[0] >= 224
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
        }
    }
}

pub async fn is_safe_endpoint_url(url_str: &str) -> bool {
    let url = match Url::parse(url_str) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => url,
        _ => return false,
    };

    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }

    let host = match url.host_str() {
        Some(host) => host.trim_matches('[').trim_matches(']').to_ascii_lowercase(),
        None => return false,
    };
    let trusted_host = configured_trusted_endpoint_hosts().contains(&host);

    // Plain HTTP is only acceptable for an endpoint the deployment explicitly
    // trusts, such as a private mesh node managed by the operator.
    if url.scheme() == "http" && !trusted_host {
        return false;
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        return trusted_host || !is_private_or_special_ip(ip);
    }

    let port = url.port_or_known_default().unwrap_or(443);
    let resolved = match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::lookup_host((host.as_str(), port)),
    )
    .await
    {
        Ok(Ok(addresses)) => addresses,
        _ => return false,
    };

    let addresses: Vec<IpAddr> = resolved.map(|address| address.ip()).collect();
    !addresses.is_empty()
        && (trusted_host || addresses.iter().all(|ip| !is_private_or_special_ip(*ip)))
}

fn same_endpoint_origin(left: &str, right: &str) -> bool {
    match (Url::parse(left), Url::parse(right)) {
        (Ok(left), Ok(right)) => left.scheme() == right.scheme()
            && left.host_str() == right.host_str()
            && left.port_or_known_default() == right.port_or_known_default(),
        _ => false,
    }
}

fn verify_token(headers: &HeaderMap, env_name: &str, label: &str) -> Result<(), (StatusCode, String)> {
    let required_token = match std::env::var(env_name) {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return Ok(()),
    };

    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if auth_header == format!("Bearer {}", required_token) || auth_header == required_token {
        Ok(())
    } else {
        Err((StatusCode::UNAUTHORIZED, format!("Accesso non autorizzato: token {} non valido.", label)))
    }
}

pub fn verify_admin_auth(headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    verify_token(headers, "NEXUS_ADMIN_TOKEN", "amministratore")
}

pub fn verify_api_auth(headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    verify_token(headers, "NEXUS_API_TOKEN", "API")
}

fn inference_rate_limit() -> usize {
    parse_inference_rate_limit(std::env::var("NEXUS_MAX_REQUESTS_PER_MINUTE").ok())
}

fn parse_inference_rate_limit(value: Option<String>) -> usize {
    value
        .and_then(|value| value.parse().ok())
        .filter(|value: &usize| *value > 0)
        .unwrap_or(120)
}

async fn enforce_inference_rate_limit(state: &AppState) -> Result<(), (StatusCode, String)> {
    let now = std::time::Instant::now();
    let window_start = now - std::time::Duration::from_secs(60);
    let mut requests = state.request_window.lock().await;

    while requests.front().is_some_and(|timestamp| *timestamp <= window_start) {
        requests.pop_front();
    }
    if requests.len() >= inference_rate_limit() {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Limite di richieste Nexus raggiunto. Riprova tra un minuto.".to_string(),
        ));
    }
    requests.push_back(now);
    Ok(())
}

fn is_loopback_bind(addr: &str) -> bool {
    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        return socket_addr.ip().is_loopback();
    }

    addr.rsplit_once(':')
        .map(|(host, _)| matches!(host.trim_matches('[').trim_matches(']'), "localhost" | "127.0.0.1" | "::1"))
        .unwrap_or(false)
}

fn ensure_network_auth_is_configured(addr: &str) -> anyhow::Result<()> {
    if is_loopback_bind(addr) {
        return Ok(());
    }

    for name in ["NEXUS_API_TOKEN", "NEXUS_ADMIN_TOKEN"] {
        if std::env::var(name).map(|value| !value.trim().is_empty()).unwrap_or(false) {
            continue;
        }
        anyhow::bail!("{} is required when NEXUS_ADDR is not a loopback address", name);
    }
    Ok(())
}

async fn list_providers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    verify_admin_auth(&headers)?;
    let list = state.providers.read().await;
    let mut masked_list = Vec::new();
    for p in list.iter() {
        let mut p_json = serde_json::to_value(p).unwrap_or_default();
        if let Some(ref k) = p.api_key {
            p_json["api_key"] = serde_json::Value::String(db::mask_api_key(k));
        }
        masked_list.push(p_json);
    }
    Ok(Json(serde_json::json!({ "providers": masked_list })))
}

async fn create_provider(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(input): Json<ProviderInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    verify_admin_auth(&headers)?;

    if !is_safe_endpoint_url(&input.base_url).await {
        return Err((StatusCode::BAD_REQUEST, "⚠️ SSRF Protection: Endpoint non valido o pericoloso.".to_string()));
    }

    let id = db::insert_provider_db(&state.db, &input).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, redact_secrets(&e.to_string())))?;

    let updated = db::load_all_providers(&state.db).await;
    let mut lock = state.providers.write().await;
    *lock = updated;

    info!("➕ Provider '{}' salvato e ricaricato a caldo", redact_secrets(&input.name));
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id, "status": "created" }))))
}

async fn delete_provider(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    verify_admin_auth(&headers)?;

    sqlx::query("DELETE FROM providers WHERE id = ?").bind(id).execute(&state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let updated = db::load_all_providers(&state.db).await;
    let mut lock = state.providers.write().await;
    *lock = updated;

    info!("🗑️ Provider id={} eliminato", id);
    Ok(Json(serde_json::json!({ "id": id, "status": "deleted" })))
}

async fn handle_get_catalog(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT provider_name, model_id, prompt_cost_per_1m, completion_cost_per_1m, context_length, is_free, capabilities, last_updated 
         FROM models_catalog ORDER BY is_free DESC, provider_name ASC, prompt_cost_per_1m ASC LIMIT 2000"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut catalog = Vec::new();
    let mut provider_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for row in rows {
        use sqlx::Row;
        let prov: String = row.get("provider_name");
        *provider_counts.entry(prov.clone()).or_insert(0) += 1;

        catalog.push(serde_json::json!({
            "provider_name": prov,
            "model_id": row.get::<String, _>("model_id"),
            "prompt_cost_per_1m": row.get::<f64, _>("prompt_cost_per_1m"),
            "completion_cost_per_1m": row.get::<f64, _>("completion_cost_per_1m"),
            "context_length": row.get::<i64, _>("context_length"),
            "is_free": row.get::<i64, _>("is_free") == 1,
            "last_updated": row.get::<String, _>("last_updated")
        }));
    }

    let mut providers_meta = Vec::new();
    for (k, count) in &provider_counts {
        let label = match k.as_str() {
            "openrouter" => "🪐 OpenRouter",
            "google_aistudio" | "google" => "♊ Google AI Studio",
            "groq" => "⚡ Groq Cloud",
            "deepseek" => "🧠 DeepSeek",
            "nvidia" => "🟢 NVIDIA NIM",
            "alibaba" => "🐉 Alibaba Qwen",
            "anthropic" => "🎨 Anthropic",
            "openai" => "🤖 OpenAI",
            "aws" => "☁️ AWS Bedrock",
            "inception" => "🔥 Inception / Fireworks",
            "agnes" => "🕊️ Agnes AI",
            "mistral" => "🌪️ Mistral AI",
            "together" => "🤝 Together AI",
            "perplexity" => "🔍 Perplexity",
            "cerebras" => "⚡ Cerebras",
            "sambanova" => "🟧 SambaNova",
            "ollama_local" => "🏠 Ollama Local",
            _ => k.as_str(),
        };
        providers_meta.push(serde_json::json!({
            "key": k,
            "label": label,
            "count": count
        }));
    }
    providers_meta.sort_by_key(|p| p["key"].as_str().unwrap_or("").to_string());

    Ok(Json(serde_json::json!({
        "count": catalog.len(),
        "catalog_providers": providers_meta,
        "catalog": catalog
    })))
}

async fn handle_sync_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    verify_admin_auth(&headers)?;
    let (or_count, google_count) = catalog::sync_all_catalogs(&state.client, &state.db).await;

    Ok(Json(serde_json::json!({
        "status": "synced",
        "openrouter_count": or_count,
        "google_count": google_count,
        "total_count": or_count + google_count
    })))
}

async fn handle_test_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    verify_admin_auth(&headers)?;
    let list = state.providers.read().await;
    let target = list.iter().find(|p| p.id == Some(id))
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "Provider non trovato".to_string()))?;

    drop(list);

    let test_req = LLMRequest {
        model: Some(target.model.clone()),
        messages: vec![
            types::Message {
                tool_call_id: None,
                tool_name: None,
                tool_calls_json: None,
                role: "user".to_string(),
                content: "Test di connettività Siliceo-Nexus. Rispondi 'OK'".to_string(),
            }
        ],
        temperature: Some(0.1),
        max_tokens: Some(50),
        tools: None,
        stream: None,
        stop: None,
    };

    let start = std::time::Instant::now();
    match adapters::dispatch_request(&state.client, &target, &test_req).await {
        Ok(res) => {
            let elapsed_ms = start.elapsed().as_millis();
            let content = res.choices.first()
                .map(|c| c.message.content.clone())
                .unwrap_or_else(|| "Nessuna risposta".to_string());

            Ok(Json(serde_json::json!({
                "success": true,
                "provider_name": target.name,
                "model_used": target.model,
                "latency_ms": elapsed_ms,
                "content": content
            })))
        }
        Err(e) => {
            let elapsed_ms = start.elapsed().as_millis();
            Ok(Json(serde_json::json!({
                "success": false,
                "provider_name": target.name,
                "latency_ms": elapsed_ms,
                "error": e.to_string()
            })))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SetModelPayload {
    pub model: String,
}

async fn handle_set_provider_model(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<SetModelPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    verify_admin_auth(&headers)?;

    sqlx::query("UPDATE providers SET model = ? WHERE id = ?")
        .bind(&payload.model)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let updated = db::load_all_providers(&state.db).await;
    let mut lock = state.providers.write().await;
    *lock = updated;

    info!("🔄 Modello provider id={} aggiornato a caldo in '{}'", id, payload.model);
    Ok(Json(serde_json::json!({ "id": id, "model": payload.model, "status": "updated" })))
}

#[derive(serde::Deserialize)]
pub struct FetchModelsPayload {
    pub base_url: String,
    pub api_key: Option<String>,
    pub provider_key: Option<String>,
    pub provider_id: Option<i64>,
    pub provider_name: Option<String>,
}

/// Inferisce la chiave catalogo (provider_name) dall'URL dell'endpoint.
/// Usata quando il frontend non fornisce provider_key, per non etichettare
/// tutti i modelli come "custom".
fn infer_provider_key(base_url: &str) -> String {
    let b = base_url.to_lowercase();
    if b.contains("openrouter.ai") { "openrouter".to_string() }
    else if b.contains("api.groq.com") { "groq".to_string() }
    else if b.contains("generativelanguage.googleapis.com") { "google_aistudio".to_string() }
    else if b.contains("integrate.api.nvidia.com") || b.contains("api.nvidia.com") { "nvidia".to_string() }
    else if b.contains("api.mistral.ai") { "mistral".to_string() }
    else if b.contains("apihub.agnes-ai.com") { "agnes".to_string() }
    else if b.contains("api.cerebras.ai") { "cerebras".to_string() }
    else if b.contains("api.inceptionlabs.ai") || b.contains("fireworks") { "fireworks".to_string() }
    else if b.contains("11434") || b.contains("ollama") { "ollama_local".to_string() }
    else if b.contains("100.98.20.76") || b.contains("beellama") { "beellama".to_string() }
    else if b.contains("api.anthropic.com") { "anthropic".to_string() }
    else if b.contains("api.openai.com") { "openai".to_string() }
    else { "custom".to_string() }
}

async fn handle_fetch_models(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<FetchModelsPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if let Err((code, msg)) = verify_admin_auth(&headers) {
        return Err((code, Json(serde_json::json!({ "success": false, "error": msg }))));
    }

    if !is_safe_endpoint_url(&payload.base_url).await {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": "⚠️ SSRF Protection: Endpoint non valido o pericoloso." }))));
    }

    let mut effective_key = payload.api_key.clone();

    if let Some(ref k) = effective_key {
        if k.contains("...") || k.contains('•') || k.trim().is_empty() {
            effective_key = None;
        }
    }

    if effective_key.is_none() {
        let list = state.providers.read().await;
        if let Some(p_id) = payload.provider_id {
            if let Some(p) = list.iter().find(|x| x.id == Some(p_id)) {
                if same_endpoint_origin(&payload.base_url, &p.base_url) {
                    effective_key = p.api_key.clone();
                }
            }
        }
        if effective_key.is_none() {
            if let Some(ref p_name) = payload.provider_name {
                if let Some(p) = list.iter().find(|x| x.name == *p_name) {
                    if same_endpoint_origin(&payload.base_url, &p.base_url) {
                        effective_key = p.api_key.clone();
                    }
                }
            }
        }
    }

    let mut clean_base = payload.base_url.trim_end_matches('/').to_string();
    if clean_base.ends_with("/v1") {
        clean_base = clean_base[..clean_base.len() - 3].to_string();
    } else if clean_base.ends_with("/v1beta") {
        clean_base = clean_base[..clean_base.len() - 7].to_string();
    } else if clean_base.ends_with("/openai") {
        clean_base = clean_base[..clean_base.len() - 7].to_string();
    }
    let clean_base = clean_base.trim_end_matches('/');

    let is_ollama = payload.base_url.contains("11434") || payload.base_url.contains("ollama");

    let url = if is_ollama {
        format!("{}/api/tags", clean_base)
    } else if payload.base_url.ends_with("/models") || payload.base_url.ends_with("/tags") {
        payload.base_url.clone()
    } else {
        format!("{}/v1/models", clean_base)
    };

    let mut req = state.client.get(&url);
    if let Some(key) = &effective_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", trimmed));
            req = req.header("api-key", trimmed);
        }
    }

    let resp = req.timeout(std::time::Duration::from_secs(12)).send().await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": redact_secrets(&format!("Impossibile connettersi all'endpoint {}: {}", url, e)) }))))?;

    if !resp.status().is_success() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": redact_secrets(&format!("Errore HTTP status {} dall'endpoint {}", resp.status(), url)) }))));
    }

    let body: serde_json::Value = resp.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "error": format!("Errore nel parsing JSON dei modelli da {}: {}", url, e) }))))?;

    let mut model_ids = Vec::new();

    if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
        for m in data {
            if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                model_ids.push(id.to_string());
            } else if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                model_ids.push(name.trim_start_matches("models/").to_string());
            }
        }
    }

    if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
        for m in models {
            if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                let clean = name.trim_start_matches("models/").to_string();
                if !model_ids.contains(&clean) {
                    model_ids.push(clean);
                }
            } else if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                let id_str = id.to_string();
                if !model_ids.contains(&id_str) {
                    model_ids.push(id_str);
                }
            } else if let Some(model) = m.get("model").and_then(|i| i.as_str()) {
                let m_str = model.to_string();
                if !model_ids.contains(&m_str) {
                    model_ids.push(m_str);
                }
            }
        }
    }

    // Fallback: se l'endpoint è un nodo Ollama (porta 11434 o 'ollama' o localhost), tenta anche /api/tags per estrarre tutti i modelli locali scaricati su disco!
    if (payload.base_url.contains("11434") || payload.base_url.contains("ollama") || payload.base_url.contains("localhost")) && !url.contains("/api/tags") {
        let mut clean_base = payload.base_url.trim_end_matches('/').to_string();
        if clean_base.ends_with("/v1") {
            clean_base = clean_base[..clean_base.len() - 3].to_string();
        } else if clean_base.ends_with("/v1beta") {
            clean_base = clean_base[..clean_base.len() - 7].to_string();
        } else if clean_base.ends_with("/openai") {
            clean_base = clean_base[..clean_base.len() - 7].to_string();
        }
        let clean_base = clean_base.trim_end_matches('/');
        let tags_url = format!("{}/api/tags", clean_base);

        if let Ok(tags_resp) = state.client.get(&tags_url).timeout(std::time::Duration::from_secs(5)).send().await {
            if tags_resp.status().is_success() {
                if let Ok(tags_body) = tags_resp.json::<serde_json::Value>().await {
                    if let Some(models) = tags_body.get("models").and_then(|m| m.as_array()) {
                        for m in models {
                            if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                                let name_str = name.to_string();
                                if !model_ids.contains(&name_str) {
                                    model_ids.push(name_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if model_ids.is_empty() {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "success": false, "error": format!("Nessun modello estratto dalla risposta di {}", url) }))));
    }

    // Se il frontend non specifica provider_key, inferiscilo dall'URL invece di
    // etichettare tutto "custom". Evita centinaia di modelli senza origine corretta.
    let prov_key = match payload.provider_key.as_deref() {
        Some(k) if !k.trim().is_empty() && k != "custom" => k.to_string(),
        _ => infer_provider_key(&payload.base_url),
    };
    for m_id in &model_ids {
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO models_catalog 
             (provider_name, model_id, prompt_cost_per_1m, completion_cost_per_1m, context_length, is_free, capabilities, last_updated)
             VALUES (?, ?, 0.0, 0.0, 131072, 1, '[\"text\", \"chat\"]', datetime('now'))"
        )
        .bind(&prov_key)
        .bind(m_id)
        .execute(&state.db)
        .await;
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "endpoint_used": url,
        "count": model_ids.len(),
        "models": model_ids
    })))
}

#[cfg(test)]
mod tests {
    use super::{is_loopback_bind, is_private_or_special_ip, parse_inference_rate_limit, same_endpoint_origin};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn detects_loopback_bindings() {
        assert!(is_loopback_bind("127.0.0.1:8082"));
        assert!(is_loopback_bind("[::1]:8082"));
        assert!(is_loopback_bind("localhost:8082"));
        assert!(!is_loopback_bind("0.0.0.0:8082"));
        assert!(!is_loopback_bind("100.64.0.10:8082"));
    }

    #[test]
    fn classifies_private_and_special_addresses() {
        assert!(is_private_or_special_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(is_private_or_special_ip(IpAddr::V4(Ipv4Addr::new(100, 98, 20, 76))));
        assert!(is_private_or_special_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))));
        assert!(!is_private_or_special_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
    }

    #[test]
    fn only_reuses_keys_for_the_same_origin() {
        assert!(same_endpoint_origin(
            "https://api.example.test/v1",
            "https://api.example.test/v1/models"
        ));
        assert!(!same_endpoint_origin(
            "https://api.example.test/v1",
            "https://attacker.example.test/v1"
        ));
    }

    #[test]
    fn uses_a_safe_default_rate_limit() {
        assert_eq!(parse_inference_rate_limit(None), 120);
        assert_eq!(parse_inference_rate_limit(Some("0".to_string())), 120);
        assert_eq!(parse_inference_rate_limit(Some("42".to_string())), 42);
    }
}
