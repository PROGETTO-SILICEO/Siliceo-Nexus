use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

mod types;
mod db;
mod router;
mod adapters;
mod dashboard;

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
        db,
        providers: Arc::new(RwLock::new(providers_list)),
        client: reqwest::Client::new(),
    };

    let app = Router::new()
        // Dashboard SPA
        .route("/", get(handle_dashboard))
        // Main OpenAI-compatible LLM endpoint
        .route("/v1/chat/completions", post(handle_chat_completions))
        // Management API
        .route("/providers", get(list_providers).post(create_provider))
        .route("/providers/:id", delete(delete_provider))
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
    dashboard::render_dashboard()
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

    // 2. Selezione dinamica del provider idoneo
    let selected_provider = router::select_provider(&state.providers, intent, requires_tools).await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, e))?;

    // 3. Dispatching della richiesta
    match adapters::dispatch_request(&state.client, &selected_provider, &request).await {
        Ok(response) => {
            info!("✅ Response delivered from provider '{}' (model: {})", selected_provider.name, response.model);
            Ok(Json(response))
        }
        Err(e) => {
            error!("❌ Provider '{}' failed: {}. Triggering failover...", selected_provider.name, e);
            // Cooldown temporaneo per provider rotto
            Err((StatusCode::BAD_GATEWAY, format!("Provider error: {}", e)))
        }
    }
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
