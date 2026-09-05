use tracing::{info, warn};
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

use crate::types::FreeProviderCatalogEntry;

/// Carica il catalogo dei 100 provider gratuiti dal file JSON statico
pub fn load_free_providers_catalog() -> Vec<FreeProviderCatalogEntry> {
    let path = "data/free_providers_catalog.json";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(catalog) = serde_json::from_str::<Vec<FreeProviderCatalogEntry>>(&content) {
            return catalog;
        }
    }
    Vec::new()
}

/// Sincronizza i provider gratuiti federati nel database:
/// 1. I provider `no_auth` (es. OpenCode, DuckDuckGo, AI Horde) vengono attivati automaticamente.
/// 2. I provider con chiave API configurata nelle variabili d'ambiente (es. GROQ_API_KEY, SAMBANOVA_API_KEY, etc.) vengono attivati.
pub async fn sync_federated_free_providers(pool: &SqlitePool) -> anyhow::Result<(usize, usize)> {
    let catalog = load_free_providers_catalog();
    if catalog.is_empty() {
        return Ok((0, 0));
    }

    let mut auto_noauth_count = 0;
    let mut auto_keyed_count = 0;

    for entry in catalog {
        let existing: Option<(i64, Option<String>, i64)> = sqlx::query_as(
            "SELECT id, api_key, enabled FROM providers WHERE name = ?"
        )
        .bind(&entry.id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        if entry.no_auth {
            // Provider a zero configurazione
            if existing.is_none() {
                let tags_json = serde_json::to_string(&entry.tags).unwrap_or_else(|_| "[]".to_string());
                let res = sqlx::query(
                    "INSERT INTO providers (name, base_url, api_key, auth_type, model, priority, tier, tags, tpm_limit, rpm_limit, max_ctx, enabled)
                     VALUES (?, ?, NULL, ?, ?, 2, 'free', ?, ?, ?, ?, 1)"
                )
                .bind(&entry.id)
                .bind(&entry.base_url)
                .bind(&entry.auth_type)
                .bind(&entry.default_model)
                .bind(&tags_json)
                .bind(entry.tpm_limit as i64)
                .bind(entry.rpm_limit as i64)
                .bind(entry.max_ctx as i64)
                .execute(pool)
                .await;

                if res.is_ok() {
                    auto_noauth_count += 1;
                }
            }
        } else if let Some(ref env_name) = entry.env_var {
            // Provider che richiede chiave API: verifica se la chiave è presente nell'ambiente
            if let Ok(key) = std::env::var(env_name) {
                let key_trimmed = key.trim();
                if !key_trimmed.is_empty() {
                    let tags_json = serde_json::to_string(&entry.tags).unwrap_or_else(|_| "[]".to_string());
                    match existing {
                        Some((id, _, enabled)) => {
                            // Aggiorna chiave e riabilita se era spento
                            let _ = sqlx::query(
                                "UPDATE providers SET api_key = ?, enabled = 1, updated_at = datetime('now') WHERE id = ?"
                            )
                            .bind(key_trimmed)
                            .bind(id)
                            .execute(pool)
                            .await;
                            if enabled == 0 {
                                auto_keyed_count += 1;
                            }
                        }
                        None => {
                            let res = sqlx::query(
                                "INSERT INTO providers (name, base_url, api_key, auth_type, model, priority, tier, tags, tpm_limit, rpm_limit, max_ctx, enabled)
                                 VALUES (?, ?, ?, ?, ?, 3, 'free', ?, ?, ?, ?, 1)"
                            )
                            .bind(&entry.id)
                            .bind(&entry.base_url)
                            .bind(key_trimmed)
                            .bind(&entry.auth_type)
                            .bind(&entry.default_model)
                            .bind(&tags_json)
                            .bind(entry.tpm_limit as i64)
                            .bind(entry.rpm_limit as i64)
                            .bind(entry.max_ctx as i64)
                            .execute(pool)
                            .await;

                            if res.is_ok() {
                                auto_keyed_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if auto_noauth_count > 0 || auto_keyed_count > 0 {
        info!("🌐 Federazione Free-Tier Siliceo: attivati {} no-auth e {} con chiave da ambiente", auto_noauth_count, auto_keyed_count);
    }

    Ok((auto_noauth_count, auto_keyed_count))
}
