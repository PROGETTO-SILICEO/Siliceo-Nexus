use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

mod types;
mod db;
mod router;
mod adapters;
mod dashboard;
mod catalog;

use types::{Provider, ProviderInput, LLMRequest, LLMResponse};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub providers: Arc<RwLock<Vec<Provider>>>,
    pub client: reqwest::Client,
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
        client: reqwest::Client::new(),
    };

    // Avvia il loop di aggiornamento catalogo modelli in background (OpenRouter 24h sync)
    catalog::spawn_catalog_sync_loop(state.client.clone(), state.db.clone());

    let app = Router::new()
        // Dashboard SPA
        .route("/", get(handle_dashboard))
        // Main OpenAI-compatible LLM endpoint
        .route("/v1/chat/completions", post(handle_chat_completions))
        // Management API
        .route("/providers", get(list_providers).post(create_provider))
        .route("/providers/:id", delete(delete_provider))
        .route("/providers/:id/test", post(handle_test_provider))
        // Catalog API
        .route("/catalog", get(handle_get_catalog))
        .route("/catalog/sync", post(handle_sync_catalog))
        // Health
        .route("/health", get(handle_health))
        .with_state(state);

    let addr = std::env::var("NEXUS_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());
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

async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "siliceo-nexus",
        "version": "0.1.0"
    }))
}

async fn handle_chat_completions(
    State(state): State<AppState>,
    Json(request): Json<LLMRequest>,
) -> Result<Json<LLMResponse>, (StatusCode, String)> {
    info!("📥 Incoming chat completions request (messages: {})", request.messages.len());

    // 1. Classificazione dell'intento in < 1ms
    let intent = router::classify_intent(&request);
    let requires_tools = request.tools.is_some();

    // 2. Ottieni tutti i provider idonei ordinati per priorità per la cascata di failover
    let eligible = router::select_eligible_providers(&state.providers, intent, requires_tools).await;
    if eligible.is_empty() {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Nessun provider LLM disponibile".to_string()));
    }

    let mut last_error = String::new();

    // 3. Cascata di Failover Sequenziale
    for p in &eligible {
        info!("🚀 Tentativo con provider '{}' (model: {})", p.name, p.model);
        match adapters::dispatch_request(&state.client, p, &request).await {
            Ok(response) => {
                info!("✅ Risposta consegnata da '{}' (model: {})", p.name, response.model);
                return Ok(Json(response));
            }
            Err(e) => {
                warn!("⚠️ Provider '{}' fallito: {}. Passo al candidato successivo...", p.name, e);
                last_error = e.to_string();
            }
        }
    }

    error!("❌ Tutti i provider della cascata sono falliti. Ultimo errore: {}", last_error);
    Err((StatusCode::BAD_GATEWAY, format!("Failover esausto. Ultimo errore: {}", last_error)))
}

async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let list = state.providers.read().await;
    Json(serde_json::json!({ "providers": *list }))
}

async fn create_provider(
    State(state): State<AppState>,
    Json(input): Json<ProviderInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let id = db::insert_provider_db(&state.db, &input).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Reload in memoria
    let updated = db::load_all_providers(&state.db).await;
    let mut lock = state.providers.write().await;
    *lock = updated;

    info!("➕ Provider '{}' salvato e ricaricato a caldo", input.name);
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id, "status": "created" }))))
}

async fn delete_provider(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
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
         FROM models_catalog ORDER BY is_free DESC, provider_name ASC, prompt_cost_per_1m ASC LIMIT 1000"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut catalog = Vec::new();
    let mut openrouter_count = 0;
    let mut google_count = 0;

    for row in rows {
        use sqlx::Row;
        let prov: String = row.get("provider_name");
        if prov == "google_aistudio" {
            google_count += 1;
        } else {
            openrouter_count += 1;
        }

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

    Ok(Json(serde_json::json!({
        "count": catalog.len(),
        "openrouter_count": openrouter_count,
        "google_count": google_count,
        "catalog": catalog
    })))
}

async fn handle_sync_catalog(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
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
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let list = state.providers.read().await;
    let target = list.iter().find(|p| p.id == Some(id))
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "Provider non trovato".to_string()))?;

    drop(list);

    let test_req = LLMRequest {
        model: Some(target.model.clone()),
        messages: vec![
            types::Message {
                role: "user".to_string(),
                content: "Test di connettività Siliceo-Nexus. Rispondi 'OK'".to_string(),
            }
        ],
        temperature: Some(0.1),
        max_tokens: Some(50),
        tools: None,
        stream: None,
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
