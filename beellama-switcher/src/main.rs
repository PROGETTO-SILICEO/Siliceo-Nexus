use axum::{
    extract::{Json, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(Clone)]
pub struct AppState {
    pub models_dir: PathBuf,
    pub beellama_bin: String,
    pub internal_port: u16,
    pub context_size: u32,
    pub default_cache_k: String,
    pub default_cache_v: String,
    pub default_flash_attn: String,
    pub active_process: Arc<Mutex<Option<tokio::process::Child>>>,
    pub active_model: Arc<Mutex<String>>,
    pub client: reqwest::Client,
}

#[derive(Serialize)]
pub struct ModelItem {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub path: String,
}

#[derive(Deserialize)]
pub struct SwitchModelRequest {
    pub model: String,
    pub cache_k: Option<String>,
    pub cache_v: Option<String>,
    pub flash_attn: Option<String>,
    pub context_size: Option<u32>,
    pub draft_model: Option<String>,
    pub slots: Option<u32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let models_dir_str = std::env::var("BEELLAMA_MODELS_DIR")
        .unwrap_or_else(|_| "/home/alforiva/inference/models".to_string());
    let models_dir = PathBuf::from(&models_dir_str);

    if !models_dir.exists() {
        let _ = std::fs::create_dir_all(&models_dir);
    }

    let beellama_bin = std::env::var("BEELLAMA_BIN")
        .unwrap_or_else(|_| "/home/alforiva/inference/beellama_src/build/bin/llama-server".to_string());
    let internal_port = std::env::var("BEELLAMA_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()
        .unwrap_or(8081);
    // Context size configurabile via env (default 8192; gemma-4-E4B vuole 65536)
    let context_size = std::env::var("BEELLAMA_CONTEXT")
        .unwrap_or_else(|_| "8192".to_string())
        .parse::<u32>()
        .unwrap_or(8192);
    let default_cache_k = std::env::var("BEELLAMA_CACHE_K")
        .unwrap_or_else(|_| "turbo4".to_string());
    let default_cache_v = std::env::var("BEELLAMA_CACHE_V")
        .unwrap_or_else(|_| "turbo3_tcq".to_string());
    let default_flash_attn = std::env::var("BEELLAMA_FLASH_ATTN")
        .unwrap_or_else(|_| "on".to_string());

    let state = AppState {
        models_dir,
        beellama_bin,
        internal_port,
        context_size,
        default_cache_k,
        default_cache_v,
        default_flash_attn,
        active_process: Arc::new(Mutex::new(None)),
        active_model: Arc::new(Mutex::new("none".to_string())),
        client: reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/v1/models", get(handle_list_models))
        .route("/models", get(handle_list_models))
        .route("/v1/switch_model", post(handle_switch_model))
        .route("/switch_model", post(handle_switch_model))
        .route("/v1/chat/completions", post(handle_proxy_chat))
        .route("/v1/completions", post(handle_proxy_chat))
        .layer(cors)
        .with_state(state);

    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    info!("🚀 BeeLlama Switcher (Rust) in ascolto su http://{}", listen_addr);
    info!("📁 Cartella Modelli GGUF/TurboQuant: {:?}", models_dir_str);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let current = state.active_model.lock().await.clone();
    Json(serde_json::json!({
        "status": "online",
        "service": "beellama-switcher-rust",
        "active_model": current,
        "internal_port": state.internal_port,
        "default_cache_k": state.default_cache_k,
        "default_cache_v": state.default_cache_v,
        "default_flash_attn": state.default_flash_attn,
        "models_dir": state.models_dir.to_string_lossy()
    }))
}

async fn handle_list_models(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut models = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&state.models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "gguf" || ext == "bin" || ext == "tq" {
                        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        models.push(serde_json::json!({
                            "id": filename,
                            "name": filename,
                            "size_bytes": size,
                            "path": path.to_string_lossy()
                        }));
                    }
                }
            }
        }
    }

    models.sort_by_key(|m| m["id"].as_str().unwrap_or("").to_string());

    Json(serde_json::json!({
        "object": "list",
        "data": models,
        "count": models.len()
    }))
}

async fn handle_switch_model(
    State(state): State<AppState>,
    Json(payload): Json<SwitchModelRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let model_name = payload.model.trim();
    if model_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Nome modello non specificato".to_string()));
    }
    // Sanitizzazione path traversal: blocca componenti pericolose
    if model_name.contains("..") || model_name.contains('/') || model_name.contains('\\') {
        return Err((StatusCode::BAD_REQUEST, "Nome modello non valido (path traversal bloccato)".to_string()));
    }

    let target_path = state.models_dir.join(model_name);
    if !target_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("File modello '{}' non trovato nella directory {:?}", model_name, state.models_dir),
        ));
    }

    info!("🔄 Arresto istanza beellama corrente in corso...");
    let mut proc_lock = state.active_process.lock().await;
    if let Some(mut child) = proc_lock.take() {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    // Fallback robusto: uccidi QUALSIASI llama-server esistente sulla porta interna.
    // Copre il caso in cui il processo precedente è stato avviato prima di questo
    // switcher (es. dopo un riavvio del servizio) e non è nel nostro Child state.
    let port = state.internal_port;
    let _ = tokio::process::Command::new("pkill")
        .arg("-f")
        .arg(format!("llama-server.*--port {}", port))
        .status()
        .await;
    // Attendi che la VRAM si liberi
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Cache richieste (payload) o default. Nota: turbo4 richiede un block size 128
    // che divide n_embd_head_k del modello (es. LFM2.5 ha 64 -> incompatibile).
    // Per questo, se il primo avvio non riesce, si ritenta con cache f16/f16.
    let base_cache_k = payload.cache_k.clone().unwrap_or_else(|| state.default_cache_k.clone());
    let base_cache_v = payload.cache_v.clone().unwrap_or_else(|| state.default_cache_v.clone());
    let req_flash_attn = payload.flash_attn.clone().unwrap_or_else(|| state.default_flash_attn.clone());
    let model_ctx = payload.context_size.unwrap_or_else(|| context_for_model(model_name, state.context_size));
    let slots = payload.slots.unwrap_or(1);

    let mut final_k = base_cache_k.clone();
    let mut final_v = base_cache_v.clone();
    let mut fallback_f16 = false;

    let mut attempt = spawn_and_wait(
        &state, &mut proc_lock, &target_path, model_name, model_ctx,
        &req_flash_attn, &base_cache_k, &base_cache_v, slots, payload.draft_model.as_deref(),
    ).await;

    if attempt.is_err() && !(base_cache_k == "f16" && base_cache_v == "f16") {
        info!("⚠️  Primo avvio non riuscito ({}): retry con cache f16/f16",
              attempt.as_ref().err().map(|s| s.as_str()).unwrap_or("?"));
        fallback_f16 = true;
        final_k = "f16".to_string();
        final_v = "f16".to_string();
        attempt = spawn_and_wait(
            &state, &mut proc_lock, &target_path, model_name, model_ctx,
            &req_flash_attn, "f16", "f16", slots, payload.draft_model.as_deref(),
        ).await;
    }

    match attempt {
        Ok(()) => {
            let mut active_lock = state.active_model.lock().await;
            *active_lock = model_name.to_string();
            info!("✅ Server pronto per '{}' (KV: {}/{}{})", model_name, final_k, final_v,
                  if fallback_f16 { ", fallback f16" } else { "" });
            Ok(Json(serde_json::json!({
                "status": "switched",
                "model": model_name,
                "path": target_path.to_string_lossy(),
                "internal_port": state.internal_port,
                "context_size": model_ctx,
                "cache_type_k": final_k,
                "cache_type_v": final_v,
                "flash_attn": req_flash_attn,
                "fallback_f16": fallback_f16
            })))
        }
        Err(why) => {
            error!("❌ Avvio fallito per '{}': {}", model_name, why);
            Err((StatusCode::BAD_GATEWAY, format!(
                "Avvio modello fallito: {}. Log server: {}",
                why, server_log_path(state.internal_port).display()
            )))
        }
    }
}

async fn handle_proxy_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", state.internal_port);

    let mut req = state.client.post(&url);
    for (k, v) in headers.iter() {
        if k != "host" && k != "content-length" {
            req = req.header(k, v);
        }
    }

    let resp = req
        .body(body)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Errore inoltro a beellama: {}", e)))?;

    let status = resp.status();

    // Pass-through in streaming: NON bufferizzare con bytes() che rompe lo SSE.
    // Se il client chiede stream, inoltriamo lo stream bytes così com'è.
    let stream = resp.bytes_stream();
    let body = axum::body::Body::from_stream(stream);
    let mut response = axum::response::Response::builder()
        .status(status)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("x-accel-buffering", "no")
        .body(body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    response.headers_mut().insert("x-accel-buffering", "no".parse().unwrap());
    Ok(response)
}

/// Context per modello. gemma-4-E4B supporta 64k; qwen2.5-coder 32k;
/// gli altri (Qwen 3.5 4B/9B/35B, wizard) restano al default della config.
/// Un context troppo alto su 8GB VRAM impedisce il load (KV cache).
fn context_for_model(model_name: &str, default_ctx: u32) -> u32 {
    let lower = model_name.to_lowercase();
    if lower.contains("gemma") || lower.contains("e4b") || lower.contains("qwen3.8") || lower.contains("distill") || lower.contains("heretic") || lower.contains("ornith") || lower.contains("defiant") {
        65536
    } else if lower.contains("qwen2.5-coder") || lower.contains("coder-7b") {
        32768
    } else {
        default_ctx
    }
}

/// Nome del file di log del server interno (stdout+stderr del llama-server).
fn server_log_path(port: u16) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("beellama-server-{}.log", port))
}

/// Avvia llama-server e attende che ascolti. Se il processo muore durante
/// l'attesa lo "reapa" subito (niente zombie) e ritorna il motivo dell'errore.
#[allow(clippy::too_many_arguments)]
async fn spawn_and_wait(
    state: &AppState,
    proc_lock: &mut tokio::sync::MutexGuard<'_, Option<tokio::process::Child>>,
    target_path: &std::path::Path,
    model_name: &str,
    model_ctx: u32,
    flash_attn: &str,
    cache_k: &str,
    cache_v: &str,
    slots: u32,
    draft_model: Option<&str>,
) -> Result<(), String> {
    info!("🚀 Avvio '{}' (ctx: {}, KV: {}/{}, FA: {}, slots: {})",
          model_name, model_ctx, cache_k, cache_v, flash_attn, slots);

    // stdout/stderr del server su file: senza questo il motivo dei crash va perso.
    let log_path = server_log_path(state.internal_port);
    let log_file = std::fs::OpenOptions::new().create(true).append(true).open(&log_path)
        .map_err(|e| format!("log non apribile ({}): {}", log_path.display(), e))?;
    let log_file2 = log_file.try_clone().map_err(|e| format!("log non duplicabile: {}", e))?;

    let mut cmd = tokio::process::Command::new(&state.beellama_bin);
    cmd.arg("--model").arg(target_path)
        .arg("--port").arg(state.internal_port.to_string())
        .arg("--host").arg("127.0.0.1")
        .arg("-ngl").arg("99")
        .arg("-c").arg(model_ctx.to_string())
        .arg("-np").arg(slots.to_string())
        .arg("-fa").arg(flash_attn)
        .stdout(std::process::Stdio::from(log_file))
        .stderr(std::process::Stdio::from(log_file2));

    if !cache_k.is_empty() && cache_k != "f16" {
        cmd.arg("-ctk").arg(cache_k);
    }
    if !cache_v.is_empty() && cache_v != "f16" {
        cmd.arg("-ctv").arg(cache_v);
    }

    if let Some(draft) = draft_model {
        let draft_path = state.models_dir.join(draft);
        if draft_path.exists() {
            cmd.arg("-md").arg(draft_path).arg("-ngld").arg("99");
        }
    }

    let mut child = cmd.spawn().map_err(|e| format!("spawn fallito: {}", e))?;

    // Attesa con sorveglianza: pronto (health HTTP ok) O morto (reap, niente zombie).
    // NB: la porta TCP si apre PRIMA del load del modello: un check solo-TCP darebbe
    // falsi positivi (load fallito ma processo vivo e porta aperta -> "Loading model").
    let health_url = format!("http://127.0.0.1:{}/health", state.internal_port);
    for _ in 0..80 {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("server terminato all'avvio ({}); vedi {}",
                                   status, log_path.display()));
            }
            Ok(None) => {}
            Err(e) => return Err(format!("try_wait: {}", e)),
        }
        if let Ok(resp) = state.client.get(&health_url).timeout(Duration::from_secs(2)).send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.text().await {
                    if body.contains("ok") {
                        **proc_lock = Some(child);
                        return Ok(());
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let _ = child.kill().await;
    let _ = child.wait().await;
    Err(format!("timeout: porta {} non pronta in 40s; vedi {}",
                state.internal_port, log_path.display()))
}
