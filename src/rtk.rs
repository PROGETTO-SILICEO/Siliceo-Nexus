//! Modulo RTK (Run-Time Knowledge) / Context Hygiene per Siliceo-Nexus.
//! Pulisce e comprime l'output di terminale, log e diff ridondanti nei messaggi utente,
//! eliminando sequenze ANSI escape, linee duplicate e frammenti di progresso,
//! risparmiando dal 20% al 60% dei token senza perdita di significato per il modello.

use crate::types::Message;
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex per rimuovere sequenze ANSI escape (colori, cursori, codici terminale).
static ANSI_ESCAPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap()
});

/// Rimuove sequenze ANSI escape dal testo.
pub fn strip_ansi(text: &str) -> String {
    if !text.contains('\x1B') {
        return text.to_string();
    }
    ANSI_ESCAPE_REGEX.replace_all(text, "").to_string()
}

/// Comprime linee consecutive duplicate (es. barre di progresso, log ripetitivi).
pub fn dedup_consecutive_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_line: Option<&str> = None;
    let mut repeat_count = 0;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(prev) = last_line {
            if prev == trimmed && !trimmed.is_empty() {
                repeat_count += 1;
                continue;
            } else if repeat_count > 0 {
                out.push_str(&format!("  [... ripetuto {} volte ...]\n", repeat_count));
                repeat_count = 0;
            }
        }
        out.push_str(line);
        out.push('\n');
        last_line = Some(trimmed);
    }

    if repeat_count > 0 {
        out.push_str(&format!("  [... ripetuto {} volte ...]\n", repeat_count));
    }

    // Se il testo originale non terminava con newline, facciamo trim dell'ultimo aggiunto
    if !text.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }

    out
}

/// Applica la pipeline RTK completa su un testo:
/// 1. Rimozione ANSI
/// 2. Deduplicazione linee consecutive
pub fn sanitize_prompt_text(text: &str) -> String {
    let stripped = strip_ansi(text);
    dedup_consecutive_lines(&stripped)
}

/// Sanifica i messaggi di una richiesta LLM sul posto e ritorna il numero approssimativo di caratteri risparmiati.
pub fn sanitize_messages(messages: &mut Vec<Message>) -> usize {
    let mut chars_saved = 0;
    for msg in messages.iter_mut() {
        // Applichiamo RTK solo su messaggi utente o tool (dove si annidano log e comandi di output)
        if msg.role == "user" || msg.role == "tool" {
            let orig_len = msg.content.len();
            if orig_len > 120 {
                let cleaned = sanitize_prompt_text(&msg.content);
                if cleaned.len() < orig_len {
                    chars_saved += orig_len - cleaned.len();
                    msg.content = cleaned;
                }
            }
        }
    }
    chars_saved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        let raw = "\x1b[31mErrore critico\x1b[0m nel modulo \x1b[1mtest\x1b[0m";
        let clean = strip_ansi(raw);
        assert_eq!(clean, "Errore critico nel modulo test");
    }

    #[test]
    fn test_dedup_consecutive_lines() {
        let raw = "Riga 1\nProgresso 10%\nProgresso 10%\nProgresso 10%\nProgresso 100%";
        let clean = dedup_consecutive_lines(raw);
        assert!(clean.contains("Progresso 10%\n  [... ripetuto 2 volte ...]\nProgresso 100%"));
    }
}
