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

    let state = AppState {
        models_dir,
        beellama_bin,
        internal_port,
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
    }

    info!("🚀 Avvio nuovo modello su RTX 2070 GPU: '{}'", model_name);
    let child_res = tokio::process::Command::new(&state.beellama_bin)
        .arg("--model")
        .arg(&target_path)
        .arg("--port")
        .arg(state.internal_port.to_string())
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-ngl")
        .arg("99")
        .arg("-c")
        .arg("8192")
        .spawn();

    match child_res {
        Ok(child) => {
            *proc_lock = Some(child);
            let mut active_lock = state.active_model.lock().await;
            *active_lock = model_name.to_string();

            info!("✅ Processo beellama riavviato con successo per '{}'", model_name);
            Ok(Json(serde_json::json!({
                "status": "switched",
                "model": model_name,
                "path": target_path.to_string_lossy(),
                "internal_port": state.internal_port
            })))
        }
        Err(e) => {
            error!("❌ Errore nell'avvio di beellama: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Impossibile avviare beellama binary '{}': {}", state.beellama_bin, e),
            ))
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
    let resp_bytes = resp
        .bytes()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Errore lettura risposta beellama: {}", e)))?;

    Ok((status, resp_bytes))
}
