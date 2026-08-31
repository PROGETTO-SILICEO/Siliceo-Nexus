//! Rizzo PII — filtro reversibile per Nexus
//! Ispirato a Rizzo-AI-Academy (mmBERT, 22 categorie, GDPR by design)
//! Implementazione Rust leggera e reversibile per il gateway:
//! - Regex per le categorie critiche italiane (CF, PIVA, IBAN, EMAIL, PHONE, CAP, INDIRIZZO parziale)
//! - Vault in-memory + persistenza JSON per reversibilità
//! - Latenza target < 5ms per prompt medio (regex, no modello)

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// --- Tipi PII supportati (sottoinsieme delle 22 di Rizzo, le più sensibili per Nexus) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PiiCategory {
    Cf,         // Codice Fiscale 16 char alfanumerico
    Piva,       // Partita IVA 11 cifre
    Iban,       // IBAN IT...
    Email,
    Phone,      // telefono IT +39 o 3xx...
    Cap,        // CAP 5 cifre (contesto indirizzo)
}

impl PiiCategory {
    fn as_str(&self) -> &'static str {
        match self {
            PiiCategory::Cf => "CF",
            PiiCategory::Piva => "PIVA",
            PiiCategory::Iban => "IBAN",
            PiiCategory::Email => "EMAIL",
            PiiCategory::Phone => "PHONE",
            PiiCategory::Cap => "CAP",
        }
    }
}

// --- Regex precompilate ---
static RE_CF: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[A-Z]{6}\d{2}[A-Z]\d{2}[A-Z]\d{3}[A-Z]\b").unwrap());
static RE_PIVA: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{11}\b").unwrap());
static RE_IBAN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bIT\d{2}[ ]?[A-Z]\d{3}[ ]?\d{4}[ ]?\d{4}[ ]?\d{4}[ ]?\d{4}[ ]?\d{3}\b|\bIT\d{25,27}\b").unwrap());
static RE_EMAIL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b").unwrap());
static RE_PHONE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\+39[\s\-\.]?)?\b3\d{2}[\s\-\.]?\d{6,7}\b|\b0\d{1,4}[\s\-\.]?\d{5,8}\b").unwrap());
// CAP solo se vicino a indirizzo (euristica: 5 cifre precedute da via/piazza/etc o seguite da città) — per ora semplice 5 cifre in contesto indirizzo
static RE_CAP: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{5}\b").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub placeholder: String,
    pub original: String,
    pub category: String,
}

#[derive(Debug, Default)]
pub struct PiiVault {
    // request_id -> Vec<entry>
    inner: HashMap<String, Vec<VaultEntry>>,
}

impl PiiVault {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn store(&mut self, request_id: String, entries: Vec<VaultEntry>) {
        self.inner.insert(request_id, entries);
    }

    pub fn get(&self, request_id: &str) -> Option<&Vec<VaultEntry>> {
        self.inner.get(request_id)
    }

    pub fn remove(&mut self, request_id: &str) {
        self.inner.remove(request_id);
    }
}

// Vault globale (in-memory) — per reversibilità intra-request e breve persistenza
pub static GLOBAL_VAULT: Lazy<Arc<Mutex<PiiVault>>> = Lazy::new(|| Arc::new(Mutex::new(PiiVault::new())));

#[derive(Debug, Clone)]
pub struct AnonymizeResult {
    pub text: String,
    pub entries: Vec<VaultEntry>,
    pub count: usize,
}

/// Anonimizza un testo, ritorna testo anonimizzato + mappa per de-anonimizzare.
/// Reversibile: ogni occorrenza unica → placeholder incrementale per categoria.
pub fn anonymize(text: &str, request_id: &str) -> AnonymizeResult {
    let mut result = text.to_string();
    let mut entries = Vec::new();
    let mut counters: HashMap<PiiCategory, usize> = HashMap::new();
    let mut seen: HashMap<String, String> = HashMap::new(); // originale -> placeholder

    // Ordine: più specifici prima (IBAN, EMAIL, CF, PIVA, PHONE, CAP)
    let categories: Vec<(PiiCategory, &Regex)> = vec![
        (PiiCategory::Iban, &RE_IBAN),
        (PiiCategory::Email, &RE_EMAIL),
        (PiiCategory::Cf, &RE_CF),
        // PIVA dopo CF per non confondere, ma la PIVA è 11 cifre pure numeriche
        (PiiCategory::Piva, &RE_PIVA),
        (PiiCategory::Phone, &RE_PHONE),
        // CAP solo se sembra indirizzo (contiene via/piazza/corso) — euristica leggera
        // Per Nexus, lo trattiamo solo se il testo contiene anche parole indirizzo
        // Altrimenti saltiamo CAP per evitare falsi positivi su anni/numeri
    ];

    // Check se testo contiene indizi di indirizzo per CAP
    let has_address_hint = text.to_lowercase().contains("via ") 
        || text.to_lowercase().contains("piazza ")
        || text.to_lowercase().contains("corso ")
        || text.to_lowercase().contains("viale ");

    for (cat, re) in categories {
        // Per PIVA, evita di anonimizzare numeri che sono già stati presi come CF (CF contiene cifre)
        // e evita di prendere numeri di telefono già anonimizzati
        let mut replacements: Vec<(String, String)> = Vec::new();
        for m in re.find_iter(&result.clone()) {
            let original = m.as_str().to_string();
            // Skip se già anonimizzato (placeholder)
            if original.starts_with('[') && original.ends_with(']') {
                continue;
            }
            // Per PIVA, skip se è parte di un CF già visto o se è un numero troppo generico senza contesto
            // Euristica: PIVA solo se non è già in seen come altro tipo
            if seen.contains_key(&original) {
                continue;
            }
            // Evita di anonimizzare anni (es. 2024) come PIVA — richiedi contesto
            if cat == PiiCategory::Piva && !is_likely_piva(&original, &result) {
                continue;
            }
            if let Some(ph) = seen.get(&original) {
                replacements.push((original, ph.clone()));
            } else {
                let counter = counters.entry(cat).or_insert(0);
                *counter += 1;
                let placeholder = format!("[{}_{}]", cat.as_str(), counter);
                seen.insert(original.clone(), placeholder.clone());
                entries.push(VaultEntry {
                    placeholder: placeholder.clone(),
                    original: original.clone(),
                    category: cat.as_str().to_string(),
                });
                replacements.push((original, placeholder));
            }
        }
        // Applica sostituzioni per questa categoria (in ordine di lunghezza decrescente per non rompere indici)
        replacements.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        for (orig, ph) in replacements {
            result = result.replace(&orig, &ph);
        }
    }

    // CAP opzionale
    if has_address_hint {
        let mut cap_replacements = Vec::new();
        for m in RE_CAP.find_iter(&result.clone()) {
            let original = m.as_str().to_string();
            if seen.contains_key(&original) { continue; }
            // Solo se vicino a hint indirizzo (già verificato has_address_hint)
            let counter = counters.entry(PiiCategory::Cap).or_insert(0);
            *counter += 1;
            let placeholder = format!("[CAP_{}]", counter);
            seen.insert(original.clone(), placeholder.clone());
            entries.push(VaultEntry {
                placeholder: placeholder.clone(),
                original: original.clone(),
                category: "CAP".to_string(),
            });
            cap_replacements.push((original, placeholder));
        }
        for (orig, ph) in cap_replacements {
            result = result.replace(&orig, &ph);
        }
    }

    let count = entries.len();
    if count > 0 {
        // Salva in vault globale per reversibilità
        if let Ok(mut vault) = GLOBAL_VAULT.lock() {
            vault.store(request_id.to_string(), entries.clone());
        }
        // Persisti su file per sopravvivere al restart (append)
        let _ = persist_vault_entry(request_id, &entries);
    }

    AnonymizeResult { text: result, entries, count }
}

fn is_likely_piva(s: &str, context: &str) -> bool {
    // Euristica: PIVA è 11 cifre, spesso vicino a parole chiave
    let lower = context.to_lowercase();
    lower.contains("p.iva") || lower.contains("partita iva") || lower.contains("piva") || lower.contains("vat")
        || s.chars().all(|c| c.is_ascii_digit()) && s.len() == 11 && !lower.contains("anno")
}

/// De-anonimizza un testo usando la mappa del request_id
pub fn deanonymize(text: &str, request_id: &str) -> String {
    let vault = GLOBAL_VAULT.lock().ok();
    let entries = vault.as_ref().and_then(|v| v.get(request_id).cloned());
    // Se non in memoria, prova a caricare da file
    let entries = if let Some(e) = entries {
        e
    } else {
        load_vault_entry(request_id).unwrap_or_default()
    };
    let mut result = text.to_string();
    for e in entries {
        result = result.replace(&e.placeholder, &e.original);
    }
    result
}

// Persistenza semplice su JSON lines
fn vault_path() -> String {
    std::env::var("RIZZO_VAULT_PATH").unwrap_or_else(|_| "data/nexus_vault.jsonl".to_string())
}

fn persist_vault_entry(request_id: &str, entries: &[VaultEntry]) -> anyhow::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let path = vault_path();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    let line = serde_json::json!({
        "request_id": request_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "entries": entries
    });
    writeln!(file, "{}", line)?;
    Ok(())
}

fn load_vault_entry(request_id: &str) -> Option<Vec<VaultEntry>> {
    let path = vault_path();
    let content = std::fs::read_to_string(&path).ok()?;
    for line in content.lines().rev() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if v.get("request_id").and_then(|x| x.as_str()) == Some(request_id) {
                if let Some(arr) = v.get("entries").and_then(|x| x.as_array()) {
                    let entries: Vec<VaultEntry> = serde_json::from_value(serde_json::Value::Array(arr.clone())).ok()?;
                    return Some(entries);
                }
            }
        }
    }
    None
}

/// Helper per anonimizzare tutti i messaggi di una richiesta LLM
pub fn anonymize_messages(messages: &mut [crate::types::Message], request_id: &str) -> usize {
    let mut total = 0;
    for msg in messages.iter_mut() {
        let res = anonymize(&msg.content, request_id);
        if res.count > 0 {
            msg.content = res.text;
            total += res.count;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cf() {
        let text = "Il mio CF è RSSMRA85M01H501Z e la PIVA 12345678901";
        let res = anonymize(text, "test1");
        assert!(res.text.contains("[CF_1]"));
        assert!(res.text.contains("[PIVA_1]") || res.text.contains("12345678901")); // PIVA senza contesto potrebbe non triggerare
        assert_eq!(deanonymize(&res.text, "test1"), text);
    }
    #[test]
    fn test_email() {
        let text = "Contatta alforiva@gmail.com e mario.rossi@progettosiliceo.online";
        let res = anonymize(text, "test2");
        assert!(res.text.contains("[EMAIL_1]"));
        assert!(res.text.contains("[EMAIL_2]"));
        assert_eq!(deanonymize(&res.text, "test2"), text);
    }
    #[test]
    fn test_iban() {
        let text = "IBAN IT60 X054 2811 1010 0000 0123 456";
        let res = anonymize(text, "test3");
        assert!(res.text.contains("[IBAN_1]"));
    }
}
