//! Modulo di resilienza euristica per Siliceo-Nexus.
//! Implementa la classificazione semantica degli errori HTTP (in particolare HTTP 429)
//! distinguendo tra rate-limit transitorio per-minute (RPM/TPM) ed esaurimento effettivo
//! delle quote giornaliere o dei crediti (RPD/TPD/Billing cap).

use axum::http::HeaderMap;
use once_cell::sync::Lazy;
use regex::Regex;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureKind {
    /// Rate limit transitorio a breve termine (es. superato limite al minuto).
    /// Il fornitore specifica o suggerisce una finestra di backoff breve.
    TransientRateLimit(Duration),

    /// Esaurimento quota a lungo termine (quota giornaliera/mensile esaurita, crediti finiti, billing cap).
    /// Il fornitore richiede un'attesa prolungata (es. rollover a mezzanotte o ricarica).
    QuotaExhausted(Duration),

    /// Credenziali non valide o scadute (HTTP 401 / 403).
    AuthFailure,

    /// Errore temporaneo del server upstream (HTTP 500, 502, 503, 504).
    ServerError,

    /// Altro codice di errore HTTP o errore di rete.
    Other(u16),
}

impl FailureKind {
    pub fn cooldown_duration(&self) -> Duration {
        match self {
            FailureKind::TransientRateLimit(d) => *d,
            FailureKind::QuotaExhausted(d) => *d,
            FailureKind::AuthFailure => Duration::from_secs(600), // 10 minuti per chiavi errate
            FailureKind::ServerError => Duration::from_secs(30),   // 30s per 5xx
            FailureKind::Other(_) => Duration::from_secs(15),
        }
    }

    pub fn is_quota_exhausted(&self) -> bool {
        matches!(self, FailureKind::QuotaExhausted(_))
    }
}

/// Pattern regex per identificare l'esaurimento della quota / crediti (non semplici rate limit per-minute).
static QUOTA_REGEXES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)daily.*(?:limit|quota)").unwrap(),
        Regex::new(r"(?i)per.?day.*limit").unwrap(),
        Regex::new(r"(?i)monthly.*(?:limit|quota)").unwrap(),
        Regex::new(r"(?i)per.?month.*limit").unwrap(),
        Regex::new(r"(?i)(?:quota.*exceed|exceed.*quota)").unwrap(),
        Regex::new(r"(?i)insufficient.*quota").unwrap(),
        Regex::new(r"(?i)billing.*cap").unwrap(),
        Regex::new(r"(?i)credit.*exhaust").unwrap(),
        Regex::new(r"(?i)out of credits").unwrap(),
        Regex::new(r"(?i)hard.?limit").unwrap(),
        Regex::new(r"(?i)plan.*limit").unwrap(),
        Regex::new(r"(?i)individual quota reached").unwrap(),
        Regex::new(r"(?i)enable overages").unwrap(),
        Regex::new(r"(?i)INSUFFICIENT_G1_CREDITS_BALANCE").unwrap(),
        Regex::new(r"(?i)resource has been exhausted").unwrap(),
        Regex::new(r"(?i)daily free allocation").unwrap(),
        Regex::new(r"(?i)have exhausted their quota").unwrap(),
        Regex::new(r#"(?i)"error"\s*:\s*"usage limit reached[.\s]*""#).unwrap(),
        Regex::new(r"(?i)organization TPD rate limit").unwrap(),
        Regex::new(r"(?i)\bTPD rate limit\b").unwrap(),
        Regex::new(r"(?i)insufficient balance").unwrap(),
    ]
});

/// Pattern per estrarre il delay dichiarato nei messaggi testuali (es. Google "please retry in 38.5s")
static RETRY_DELAY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:please retry in|retry after|retry in)\s*(\d+(?:\.\d+)?)\s*s").unwrap()
});

/// Classifica un errore HTTP e ne determina la natura semantica.
pub fn classify_error(status: u16, headers: Option<&HeaderMap>, body_text: &str) -> FailureKind {
    match status {
        401 | 403 => FailureKind::AuthFailure,
        429 => classify_429(headers, body_text),
        500..=504 => FailureKind::ServerError,
        other => FailureKind::Other(other),
    }
}

/// Analizza un HTTP 429 distinguendo tra rate-limit transitorio al minuto e quota esaurita.
fn classify_429(headers: Option<&HeaderMap>, body_text: &str) -> FailureKind {
    // 1. L'indicazione esplicita del delay (header Retry-After o google.rpc.RetryInfo)
    // è il segnale autorevole prioritario: Google Gemini invia spesso preamboli tipo
    // "Resource has been exhausted (check quota)" anche per normali limitazioni al minuto di 30-40s.
    if let Some(h) = headers {
        if let Some(val) = h.get("retry-after").and_then(|v| v.to_str().ok()) {
            if let Ok(secs) = val.trim().parse::<u64>() {
                if secs >= 3600 {
                    return FailureKind::QuotaExhausted(Duration::from_secs(secs));
                }
                return FailureKind::TransientRateLimit(Duration::from_secs(secs.clamp(5, 300)));
            }
        }
    }

    // 2. Verifica se nel body c'è un hint esplicito di delay (Google RetryInfo o "please retry in Xs")
    if let Some(delay_secs) = extract_retry_delay_from_body(body_text) {
        if delay_secs >= 3600.0 {
            return FailureKind::QuotaExhausted(Duration::from_secs(delay_secs as u64));
        }
        let secs = (delay_secs.ceil() as u64).clamp(5, 300);
        return FailureKind::TransientRateLimit(Duration::from_secs(secs));
    }

    // 3. Se non c'è un delay esplicito a breve termine, controlla se è esaurimento quota effettivo
    if looks_like_quota_exhausted(body_text) {
        return FailureKind::QuotaExhausted(Duration::from_secs(3600));
    }

    // 4. Default per rate limit transitorio al minuto se non specificato altrimenti
    FailureKind::TransientRateLimit(Duration::from_secs(60))
}

fn looks_like_quota_exhausted(body: &str) -> bool {
    if body.is_empty() {
        return false;
    }
    QUOTA_REGEXES.iter().any(|re| re.is_match(body))
}

fn extract_retry_delay_from_body(body: &str) -> Option<f64> {
    if body.is_empty() {
        return None;
    }

    // Pattern testuale human-readable
    if let Some(caps) = RETRY_DELAY_REGEX.captures(body) {
        if let Some(m) = caps.get(1) {
            if let Ok(val) = m.as_str().parse::<f64>() {
                return Some(val);
            }
        }
    }

    // Pattern JSON strutturato per google.rpc.RetryInfo
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // Se c'è un campo details array contenente RetryInfo
        if let Some(details) = json.pointer("/error/details").and_then(|d| d.as_array()) {
            for item in details {
                if let Some(type_str) = item.get("@type").and_then(|t| t.as_str()) {
                    if type_str.contains("RetryInfo") {
                        if let Some(delay_str) = item.get("retryDelay").and_then(|d| d.as_str()) {
                            let clean = delay_str.trim_end_matches('s');
                            if let Ok(val) = clean.parse::<f64>() {
                                return Some(val);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_classify_quota_exhausted() {
        let body = r#"{"error":{"message":"You exceeded your current quota, please check your plan and billing details. Daily limit reached."}}"#;
        let kind = classify_error(429, None, body);
        assert!(kind.is_quota_exhausted());
        assert_eq!(kind.cooldown_duration(), Duration::from_secs(3600));
    }

    #[test]
    fn test_classify_google_retry_delay() {
        let body = r#"{"error":{"code":429,"message":"Resource has been exhausted (e.g. check quota). Please retry in 38.922534355s.","details":[{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"38.922534355s"}]}}"#;
        // In questo caso, il body contiene "please retry in 38.9s", quindi è un rate limit temporaneo di 39s
        let kind = classify_error(429, None, body);
        match kind {
            FailureKind::TransientRateLimit(d) => assert_eq!(d.as_secs(), 39),
            _ => panic!("Expected TransientRateLimit, got {:?}", kind),
        }
    }

    #[test]
    fn test_classify_retry_after_header() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", "45".parse().unwrap());
        let body = r#"{"error":"Rate limit reached for requests"}"#;
        let kind = classify_error(429, Some(&headers), body);
        match kind {
            FailureKind::TransientRateLimit(d) => assert_eq!(d.as_secs(), 45),
            _ => panic!("Expected TransientRateLimit, got {:?}", kind),
        }
    }

    #[test]
    fn test_classify_auth_failure() {
        let kind = classify_error(401, None, "Unauthorized");
        assert_eq!(kind, FailureKind::AuthFailure);
        assert_eq!(kind.cooldown_duration(), Duration::from_secs(600));
    }
}
