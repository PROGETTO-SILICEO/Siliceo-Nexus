use tracing::{info, warn, error};
use sqlx::SqlitePool;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModelItem>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelItem {
    id: String,
    name: Option<String>,
    context_length: Option<u32>,
    pricing: Option<OpenRouterPricing>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: Option<String>,
    completion: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleModelsResponse {
    models: Vec<GoogleModelItem>,
}

#[derive(Debug, Deserialize)]
struct GoogleModelItem {
    name: String,
    #[serde(rename = "displayName")]
    _display_name: Option<String>,
    #[serde(rename = "inputTokenLimit")]
    input_token_limit: Option<u32>,
}

/// Scarica ed aggiorna la tabella models_catalog con i dati ufficiali di OpenRouter
pub async fn sync_openrouter_catalog(client: &reqwest::Client, pool: &SqlitePool) -> anyhow::Result<usize> {
    info!("🔄 Download catalogo modelli aggiornato da OpenRouter...");

    let resp = client.get("https://openrouter.ai/api/v1/models")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("OpenRouter models API error status: {}", resp.status());
    }

    let catalog_data: OpenRouterModelsResponse = resp.json().await?;
    let mut updated_count = 0;

    let mut tx = pool.begin().await?;

    for item in catalog_data.data {
        let prompt_cost_1m = item.pricing.as_ref()
            .and_then(|p| p.prompt.as_deref())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|c| c * 1_000_000.0)
            .unwrap_or(0.0);

        let completion_cost_1m = item.pricing.as_ref()
            .and_then(|p| p.completion.as_deref())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|c| c * 1_000_000.0)
            .unwrap_or(0.0);

        let is_free = prompt_cost_1m == 0.0 && completion_cost_1m == 0.0;
        let context_len = item.context_length.unwrap_or(32768);

        let res = sqlx::query(
            "INSERT OR REPLACE INTO models_catalog 
             (provider_name, model_id, prompt_cost_per_1m, completion_cost_per_1m, context_length, is_free, capabilities, last_updated)
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind("openrouter")
        .bind(&item.id)
        .bind(prompt_cost_1m)
        .bind(completion_cost_1m)
        .bind(context_len as i64)
        .bind(if is_free { 1i64 } else { 0i64 })
        .bind("[\"text\", \"chat\"]")
        .execute(&mut *tx)
        .await;

        if res.is_ok() {
            updated_count += 1;
        }
    }

    tx.commit().await?;

    info!("✅ Catalogo OpenRouter aggiornato nel DB: {} modelli", updated_count);
    Ok(updated_count)
}

/// Scarica ed aggiorna la tabella models_catalog con i modelli ufficiali di Google AI Studio
pub async fn sync_google_catalog(client: &reqwest::Client, pool: &SqlitePool) -> anyhow::Result<usize> {
    info!("🔄 Download catalogo modelli aggiornato da Google AI Studio...");

    // Cerca la chiave Gemini dalle env o dal database dei provider
    let mut api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT api_key FROM providers WHERE name = 'gemini-free-tier' OR base_url LIKE '%generativelanguage.googleapis.com%' LIMIT 1"
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        if let Some((Some(k),)) = row {
            api_key = k;
        }
    }

    if api_key.is_empty() {
        warn!("⚠️ Nessuna GEMINI_API_KEY trovata per il sync del catalogo Google AI Studio.");
        return Ok(0);
    }

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key);
    let resp = client.get(&url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Google AI Studio models API error status: {}", resp.status());
    }

    let catalog_data: GoogleModelsResponse = resp.json().await?;
    let mut updated_count = 0;

    let mut tx = pool.begin().await?;

    for item in catalog_data.models {
        let clean_id = item.name.trim_start_matches("models/").to_string();
        let context_len = item.input_token_limit.unwrap_or(1048576);

        let res = sqlx::query(
            "INSERT OR REPLACE INTO models_catalog 
             (provider_name, model_id, prompt_cost_per_1m, completion_cost_per_1m, context_length, is_free, capabilities, last_updated)
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind("google_aistudio")
        .bind(&clean_id)
        .bind(0.0f64) // Free tier default per studio
        .bind(0.0f64)
        .bind(context_len as i64)
        .bind(1i64) // Free tier
        .bind("[\"text\", \"chat\", \"multimodal\"]")
        .execute(&mut *tx)
        .await;

        if res.is_ok() {
            updated_count += 1;
        }
    }

    tx.commit().await?;

    info!("✅ Catalogo Google AI Studio aggiornato nel DB: {} modelli", updated_count);
    Ok(updated_count)
}

/// Sincronizza tutti i cataloghi supportati (OpenRouter + Google AI Studio)
pub async fn sync_all_catalogs(client: &reqwest::Client, pool: &SqlitePool) -> (usize, usize) {
    let or_count = sync_openrouter_catalog(client, pool).await.unwrap_or(0);
    let goog_count = sync_google_catalog(client, pool).await.unwrap_or(0);
    (or_count, goog_count)
}

/// Task in background che esegue il refresh del catalogo all'avvio e ogni 24 ore
pub fn spawn_catalog_sync_loop(client: reqwest::Client, pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            let (or, goog) = sync_all_catalogs(&client, &pool).await;
            info!("📊 Background Catalog Sync completato: OpenRouter={}, Google={}", or, goog);
            // Attende 24 ore prima del prossimo sync
            tokio::time::sleep(std::time::Duration::from_secs(86400)).await;
        }
    });
}
