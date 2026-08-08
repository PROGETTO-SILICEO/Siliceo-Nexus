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

/// Scarica ed aggiorna la tabella models_catalog con i dati ufficiali di OpenRouter (costi per 1M token, context size, etc.)
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
        .execute(pool)
        .await;

        if res.is_ok() {
            updated_count += 1;
        }
    }

    info!("✅ Catalogo modelli aggiornato nel DB: {} modelli registrati", updated_count);
    Ok(updated_count)
}

/// Task in background che esegue il refresh del catalogo all'avvio e ogni 24 ore
pub fn spawn_catalog_sync_loop(client: reqwest::Client, pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = sync_openrouter_catalog(&client, &pool).await {
                error!("❌ Sincronizzazione catalogo modelli fallita: {}", e);
            }
            // Attende 24 ore prima del prossimo sync
            tokio::time::sleep(std::time::Duration::from_secs(86400)).await;
        }
    });
}
