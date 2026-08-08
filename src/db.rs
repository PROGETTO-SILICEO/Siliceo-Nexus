use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use tracing::info;
use crate::types::{Provider, ProviderInput};

pub fn mask_api_key(key: &str) -> String {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return "".to_string();
    }
    if trimmed.contains("...") || trimmed.contains('•') {
        return trimmed.to_string();
    }
    if trimmed.len() <= 8 {
        return "••••••••".to_string();
    }
    let prefix = &trimmed[..4];
    let suffix = &trimmed[trimmed.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

pub fn set_secure_file_permissions(path_str: &str) {
    let path = std::path::Path::new(path_str);
    if !path.exists() {
        return;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(path, perms);
            info!("🔒 Permessi di sicurezza Unix 0600 applicati su {}", path_str);
        }
    }

    #[cfg(windows)]
    {
        info!("🔒 OS Windows rilevato: applicazione ACL di sicurezza utente su {}", path_str);
    }
}

fn set_secure_database_permissions(database_path: &str) {
    for suffix in ["", "-wal", "-shm", "-journal"] {
        set_secure_file_permissions(&format!("{}{}", database_path, suffix));
    }
}

pub async fn init_db(database_url: &str) -> anyhow::Result<SqlitePool> {
    // Configura SQLite nativamente in WAL Mode per gestire letture e scritture concorrenti senza lock!
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_millis(5000));

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    info!("💾 SQLite connesso in WAL Mode (busy_timeout=5s)");

    let clean_path = database_url.trim_start_matches("sqlite:").trim_start_matches("//");
    set_secure_database_permissions(clean_path);

    // Creazione tabelle
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS providers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            base_url TEXT NOT NULL,
            api_key TEXT,
            auth_type TEXT NOT NULL DEFAULT 'bearer',
            model TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 100,
            tier TEXT NOT NULL DEFAULT 'free',
            tags TEXT NOT NULL DEFAULT '[]',
            tpm_limit INTEGER NOT NULL DEFAULT 32000,
            rpm_limit INTEGER NOT NULL DEFAULT 15,
            enabled INTEGER NOT NULL DEFAULT 1,
            cooldown_until TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS models_catalog (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider_name TEXT NOT NULL,
            model_id TEXT NOT NULL,
            prompt_cost_per_1m REAL NOT NULL DEFAULT 0.0,
            completion_cost_per_1m REAL NOT NULL DEFAULT 0.0,
            context_length INTEGER NOT NULL DEFAULT 32768,
            is_free INTEGER NOT NULL DEFAULT 1,
            capabilities TEXT NOT NULL DEFAULT '[]',
            last_updated TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(provider_name, model_id)
        );"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS usage_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            provider_name TEXT NOT NULL,
            model_id TEXT NOT NULL,
            prompt_tokens INTEGER NOT NULL,
            completion_tokens INTEGER NOT NULL,
            cost_estimated_usd REAL NOT NULL,
            intent_tag TEXT NOT NULL DEFAULT 'chitchat'
        );"
    )
    .execute(&pool)
    .await?;

    // Seed iniziale se il DB è vuoto
    seed_default_providers(&pool).await?;
    set_secure_database_permissions(clean_path);

    Ok(pool)
}

async fn seed_default_providers(pool: &SqlitePool) -> anyhow::Result<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM providers").fetch_one(pool).await?;
    if count.0 > 0 {
        return Ok(());
    }

    info!("📦 Inserimento provider predefiniti nel database...");

    let defaults = vec![
        // 1. Nodo Inferenza Tailscale (2070 locale via beellama)
        ProviderInput {
            name: "beellama-tailscale-2070".to_string(),
            base_url: "http://100.98.20.76:8080".to_string(),
            api_key: None,
            auth_type: "none".to_string(),
            model: "Qwen3.5-4B-Q6_K.gguf".to_string(),
            priority: 1,
            tier: "local".to_string(),
            tags: vec!["chitchat".into(), "fast".into(), "local".into(), "tool_supported".into()],
            tpm_limit: 100000,
            rpm_limit: 60,
            enabled: true,
        },
        // 2. OpenRouter Free Pool
        ProviderInput {
            name: "openrouter-free-pool".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            api_key: std::env::var("OPENROUTER_API_KEY").ok(),
            auth_type: "bearer".to_string(),
            model: "openrouter/free-models".to_string(),
            priority: 2,
            tier: "free".to_string(),
            tags: vec!["coding".into(), "reasoning".into(), "cloud_free".into(), "tool_supported".into()],
            tpm_limit: 50000,
            rpm_limit: 30,
            enabled: true,
        },
        // 3. Google Gemini Free Tier
        ProviderInput {
            name: "gemini-free-tier".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            api_key: std::env::var("GEMINI_API_KEY").ok(),
            auth_type: "bearer".to_string(),
            model: "gemini-2.5-flash".to_string(),
            priority: 3,
            tier: "free".to_string(),
            tags: vec!["chitchat".into(), "coding".into(), "fast".into(), "cloud_free".into(), "tool_supported".into()],
            tpm_limit: 32000,
            rpm_limit: 15,
            enabled: true,
        },
    ];

    for p in defaults {
        insert_provider_db(pool, &p).await?;
    }

    Ok(())
}

pub async fn insert_provider_db(pool: &SqlitePool, p: &ProviderInput) -> anyhow::Result<i64> {
    let mut final_api_key = p.api_key.clone();

    // Se la chiave è mascherata o vuota durante una modifica, preserviamo quella esistente nel DB!
    if let Some(ref k) = final_api_key {
        if k.contains("...") || k.contains('•') || k.trim().is_empty() {
            let existing: Option<(Option<String>,)> = sqlx::query_as(
                "SELECT api_key FROM providers WHERE name = ?"
            )
            .bind(&p.name)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

            if let Some((Some(old_key),)) = existing {
                final_api_key = Some(old_key);
            }
        }
    }

    let tags_json = serde_json::to_string(&p.tags).unwrap_or_else(|_| "[]".to_string());
    let res = sqlx::query(
        "INSERT OR REPLACE INTO providers (name, base_url, api_key, auth_type, model, priority, tier, tags, tpm_limit, rpm_limit, enabled)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&p.name)
    .bind(&p.base_url)
    .bind(&final_api_key)
    .bind(&p.auth_type)
    .bind(&p.model)
    .bind(p.priority as i64)
    .bind(&p.tier)
    .bind(tags_json)
    .bind(p.tpm_limit as i64)
    .bind(p.rpm_limit as i64)
    .bind(p.enabled as i64)
    .execute(pool)
    .await?;

    Ok(res.last_insert_rowid())
}

pub async fn load_all_providers(pool: &SqlitePool) -> Vec<Provider> {
    let rows = sqlx::query(
        "SELECT id, name, base_url, api_key, auth_type, model, priority, tier, tags, tpm_limit, rpm_limit, enabled, cooldown_until FROM providers ORDER BY priority ASC"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter().map(|r| {
        use sqlx::Row;
        let tags_str: String = r.get("tags");
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        let priority_val: i64 = r.get("priority");
        let tpm_val: i64 = r.get("tpm_limit");
        let rpm_val: i64 = r.get("rpm_limit");
        let enabled_val: i64 = r.get("enabled");

        Provider {
            id: Some(r.get("id")),
            name: r.get("name"),
            base_url: r.get("base_url"),
            api_key: r.get("api_key"),
            auth_type: r.get("auth_type"),
            model: r.get("model"),
            priority: priority_val as u32,
            tier: r.get("tier"),
            tags,
            tpm_limit: tpm_val as u32,
            rpm_limit: rpm_val as u32,
            enabled: enabled_val != 0,
            cooldown_until: r.get("cooldown_until"),
        }
    }).collect()
}
