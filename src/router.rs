use tracing::{info, warn};
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

/// Seleziona il miglior provider disponibile basandosi sull'intento e sulle capability dinamiche
pub async fn select_provider(
    providers: &Arc<RwLock<Vec<Provider>>>,
    intent: IntentTag,
    requires_tools: bool,
) -> Result<Provider, String> {
    let list = providers.read().await;
    let required_tag = intent.as_str();

    info!("🎯 Intent classificato: '{}' (requires_tools: {})", required_tag, requires_tools);

    // Cerca prima tra i provider abilitati non in cooldown che possiedono il tag dell'intento
    for p in list.iter() {
        if !p.enabled {
            continue;
        }

        // Verifica Cooldown
        if let Some(ref cooldown) = p.cooldown_until {
            if let Ok(until) = chrono::DateTime::parse_from_rfc3339(cooldown) {
                if chrono::Utc::now() < until {
                    continue; // In cooldown
                }
            }
        }

        // Se sono richiesti tool, il provider deve avere il tag tool_supported
        if requires_tools && !p.tags.contains(&"tool_supported".to_string()) {
            continue;
        }

        // Se ha il tag richiesto per l'intento, o se è un fallback generico
        if p.tags.contains(&required_tag.to_string()) || p.tags.contains(&"general".to_string()) {
            info!("✅ Selezionato provider '{}' (model: {}) per intent '{}'", p.name, p.model, required_tag);
            return Ok(p.clone());
        }
    }

    // Fallback: prendi il primo provider abilitato disponibile
    for p in list.iter() {
        if p.enabled {
            warn!("⚠️ Nessun provider specifico per tag '{}'. Uso fallback '{}'", required_tag, p.name);
            return Ok(p.clone());
        }
    }

    Err("Nessun provider LLM attivo o disponibile in Siliceo-Nexus".to_string())
}
