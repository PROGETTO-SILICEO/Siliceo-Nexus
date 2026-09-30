use tracing::info;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::{Provider, LLMRequest};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntentTag {
    Chitchat,
    Coding,
    Reasoning,
    ToolCall,
}

impl IntentTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentTag::Chitchat => "chitchat",
            IntentTag::Coding => "coding",
            IntentTag::Reasoning => "reasoning",
            IntentTag::ToolCall => "tool_call",
        }
    }
}

/// Classifica l'intento del messaggio in entrata in < 1ms usando regole euristiche deterministiche
pub fn classify_intent(request: &LLMRequest) -> IntentTag {
    // 1. Se il client invia tools o function calling esplicita -> ToolCall
    if request.tools.is_some() {
        return IntentTag::ToolCall;
    }

    let last_user_msg = request.messages.iter().rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .unwrap_or("");

    let msg_lower = last_user_msg.to_lowercase();

    // 2. Rilevamento sintassi o parole chiave da CODICE
    let code_keywords = [
        "```", "fn ", "def ", "impl ", "struct ", "class ", "pub ", "return ",
        "cargo ", "import ", "const ", "let ", "var ", "function", "std::",
        "traceback", "error:", "exception", "bug", "refactor", "code"
    ];

    if code_keywords.iter().any(|k| msg_lower.contains(k)) {
        return IntentTag::Coding;
    }

    // 3. Rilevamento REASONING / ARCHITETTURA / ETIICA
    let reasoning_keywords = [
        "architettura", "tribunale", "analisi", "pianifica", "strategia",
        "spiega la differenza", "valuta", "confronto", "perché"
    ];

    if reasoning_keywords.iter().any(|k| msg_lower.contains(k)) {
        return IntentTag::Reasoning;
    }

    // 4. Se il messaggio è breve (< 80 caratteri) o salutatorio -> Chitchat
    let greeting_keywords = ["ciao", "buongiorno", "buonasera", "grazie", "come va", "chi sei", "presenza"];
    if last_user_msg.len() < 80 || greeting_keywords.iter().any(|g| msg_lower.contains(g)) {
        return IntentTag::Chitchat;
    }

    // Default per messaggi generici estesi
    IntentTag::Reasoning
}

/// Seleziona la lista ordinata dei provider idonei per la cascata di failover.
/// `mem_cooldowns` è la mappa in-memory dei cooldown attivi (prioritaria sul campo DB).
pub async fn select_eligible_providers(
    providers: &Arc<RwLock<Vec<Provider>>>,
    mem_cooldowns: Option<&Arc<RwLock<std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>>>>,
    intent: IntentTag,
    requires_tools: bool,
    est_tokens: usize,
) -> Vec<Provider> {
    let list = providers.read().await;
    let required_tag = intent.as_str();
    let now = chrono::Utc::now();

    info!("🎯 Intent classificato: '{}' (requires_tools: {}, est_tokens: {})", required_tag, requires_tools, est_tokens);

    let mem_map: std::collections::HashMap<String, chrono::DateTime<chrono::Utc>> = match mem_cooldowns {
        Some(m) => m.read().await.clone(),
        None => std::collections::HashMap::new(),
    };

    let in_cooldown = |p: &Provider| -> bool {
        if let Some(until) = mem_map.get(&p.name) {
            if now < *until { return true; }
        }
        if let Some(ref cooldown) = p.cooldown_until {
            if let Ok(until) = chrono::DateTime::parse_from_rfc3339(cooldown) {
                if now < until { return true; }
            }
        }
        false
    };

    let mut eligible = Vec::new();

    // 1. Aggiungi provider con tag specifico o general
    for p in list.iter() {
        if !p.enabled { continue; }
        if in_cooldown(p) { continue; }
        if requires_tools && !p.tags.contains(&"tool_supported".to_string()) { continue; }
        // REVIEW 31/08: escludi i provider che non possono contenere il prompt
        if est_tokens > 0 && (p.max_ctx as usize) < est_tokens { continue; }

        if p.tags.contains(&required_tag.to_string()) || p.tags.contains(&"general".to_string()) {
            eligible.push(p.clone());
        }
    }

    // 2. Aggiungi i rimanenti abilitati per il fallback (rispettando requires_tools)
    for p in list.iter() {
        if !p.enabled { continue; }
        if eligible.iter().any(|e| e.name == p.name) { continue; }
        if in_cooldown(p) { continue; }
        if requires_tools && !p.tags.contains(&"tool_supported".to_string()) { continue; }
        if est_tokens > 0 && (p.max_ctx as usize) < est_tokens { continue; }
        eligible.push(p.clone());
    }

    eligible
}

/// PIN MODELLO (mandato Alfonso 30/09/2026): se la richiesta esplicita un
/// modello (≠ "auto"/vuoto), i provider che lo servono vengono messi in testa
/// alla cascata. Il resto resta disponibile come fallback se il pin fallisce:
/// la richiesta non può mai restare senza risposta.
pub fn pin_model_first(eligible: Vec<Provider>, requested: Option<&str>) -> Vec<Provider> {
    let want = match requested {
        Some(m) => {
            let m = m.trim();
            if m.is_empty() || m.eq_ignore_ascii_case("auto") {
                return eligible;
            }
            m.to_lowercase()
        }
        None => return eligible,
    };

    let mut pinned: Vec<Provider> = Vec::new();
    let mut rest: Vec<Provider> = Vec::new();
    for p in eligible {
        if p.model.trim().to_lowercase() == want {
            pinned.push(p);
        } else {
            rest.push(p);
        }
    }

    if !pinned.is_empty() {
        info!(
            "📌 Pin modello '{}': {} provider in testa, cascata di fallback: {}",
            want,
            pinned.len(),
            rest.len()
        );
    }
    pinned.extend(rest);
    pinned
}

/// Seleziona il miglior provider singolo disponibile
pub async fn select_provider(
    providers: &Arc<RwLock<Vec<Provider>>>,
    mem_cooldowns: Option<&Arc<RwLock<std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>>>>,
    intent: IntentTag,
    requires_tools: bool,
) -> Result<Provider, String> {
    let eligible = select_eligible_providers(providers, mem_cooldowns, intent, requires_tools, 0).await;
    if let Some(first) = eligible.first() {
        info!("✅ Selezionato provider primario '{}' (model: {})", first.name, first.model);
        Ok(first.clone())
    } else {
        Err("Nessun provider LLM attivo o disponibile in Siliceo-Nexus".to_string())
    }
}

#[cfg(test)]
mod pin_tests {
    use super::*;

    fn provider(name: &str, model: &str) -> Provider {
        Provider {
            id: None,
            name: name.into(),
            base_url: "http://x".into(),
            api_key: None,
            auth_type: "bearer".into(),
            model: model.into(),
            priority: 1,
            tier: "free".into(),
            tags: vec![],
            tpm_limit: 0,
            rpm_limit: 0,
            max_ctx: 32_000,
            enabled: true,
            cooldown_until: None,
        }
    }

    #[test]
    fn pin_porta_il_modello_richiesto_in_testa() {
        let c = vec![
            provider("a", "model-x"),
            provider("b", "gemini-3.5-flash-lite"),
            provider("c", "model-y"),
        ];
        let out = pin_model_first(c, Some("gemini-3.5-flash-lite"));
        assert_eq!(out[0].name, "b");
        assert_eq!(out.len(), 3, "la cascata completa deve restare come fallback");
    }

    #[test]
    fn pin_auto_o_assente_non_cambia_nulla() {
        let c = vec![provider("a", "model-x"), provider("b", "model-y")];
        let out1 = pin_model_first(c.clone(), Some("auto"));
        let out2 = pin_model_first(c.clone(), None);
        let out3 = pin_model_first(c.clone(), Some("   "));
        assert_eq!(out1[0].name, "a");
        assert_eq!(out2[0].name, "a");
        assert_eq!(out3[0].name, "a");
        assert_eq!(out1.len(), 2);
    }

    #[test]
    fn pin_modello_inesistente_lascia_la_cascata_intatta() {
        let c = vec![provider("a", "model-x"), provider("b", "model-y")];
        let out = pin_model_first(c, Some("non-existent"));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].name, "a");
    }

    #[test]
    fn pin_case_insensitive_e_trim() {
        let c = vec![
            provider("a", "model-x"),
            provider("b", "Gemini-3.5-Flash-Lite"),
        ];
        let out = pin_model_first(c, Some("  GEMINI-3.5-FLASH-LITE "));
        assert_eq!(out[0].name, "b");
    }
}
