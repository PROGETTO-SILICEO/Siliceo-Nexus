# 📓 Diario dei Lavori — Siliceo-Nexus

Questo documento registra in modo cronologico e rigoroso l'evoluzione architetturale di Siliceo-Nexus, in conformità con il Principio 7 (Tracciabilità Architetturale) e il Principio 8 (Empirical Enforcement) dell'Encore Protocol.

---

## [2026-09-05] — Integrazione Strategie OmniRoute & Roster 100 Provider Free-Tier

### 🎯 Obiettivo della Sessione
Evoluzione di Siliceo-Nexus verso la parità e superiorità funzionale rispetto a OmniRoute (v3.8.50), preservando la massima efficienza del kernel Rust (~10 MB di RAM, latenza sub-millisecondo):
1. **Classificatore Euristico 429 (`src/resilience.rs`)**: distinzione tra rate-limit transitorio al minuto e vera esaurimento di quota giornaliera/mensile con parsing di `Retry-After` e Google `RetryInfo`.
2. **Multi-Key Intra-Pool Fallback**: rotazione intelligente delle chiavi all'interno dello stesso provider su 429 transitorio prima di escalare al provider successivo.
3. **Catalogo Federato dei 100 Provider Free-Tier (`data/free_providers_catalog.json`)**: auto-attivazione dei provider zero-auth (OpenCode, DuckDuckGo, AI Horde) e auto-rilevamento dinamico basato sulle chiavi API in `.env`.
4. **Modulo RTK Context Hygiene (`src/rtk.rs`)**: pulizia di ANSI escape, diff ridondanti e log nei prompt per massimizzare il risparmio di token gratuiti.
5. **Dashboard & Telemetria Evoluta (`src/dashboard.rs`)**: monitoraggio grafico del pool di 100 provider, timer di cooldown semantici e contatore dei token gratuiti consumati.

### 📝 Registro Modifiche & Implementazione
- Inizializzato `DIARIO_LAVORI.md`.
- Estratto il catalogo dei 100 provider gratuiti verificati dal repository di OmniRoute in `data/free_providers_catalog.json`.
- Implementato `src/resilience.rs`:
  - Enum `FailureKind` (`TransientRateLimit`, `QuotaExhausted`, `AuthFailure`, `InternalError`, `ServiceUnavailable`).
  - Parsing semantico avanzato di HTTP headers (`Retry-After`), Google `google.rpc.RetryInfo` (`retryDelay`) e regex su pattern di quota (`daily.*limit`, `billing.*cap`, `insufficient.*quota`).
  - Test unitari completi eseguiti con successo.
- Implementato `src/rtk.rs`:
  - `strip_ansi` per la rimozione di codici di escape ANSI terminale da log e comandi.
  - `dedup_consecutive_lines` per compattare log ripetuti e ridurre token sprecati.
  - `sanitize_messages` e `sanitize_prompt_text` integrati all'ingresso dei percorsi `/v1/chat/completions` e `/v1/messages`.
- Aggiornato `src/adapters.rs`:
  - `resolve_provider_keys` e `pick_ordered_keys` per estrarre chiavi multiple (`NAME_API_KEY`, `NAME_API_KEY_2`, `NAME_KEYS="k1,k2"`).
  - Ciclo di failover intra-pool su HTTP 429: prova prima le chiavi alternative dello stesso provider prima di escalare il guasto al router globale.
- Aggiornato `src/catalog.rs`:
  - `sync_federated_free_providers`: auto-attivazione no-auth (6 provider) e auto-attivazione key-driven via env (`.env`).
  - Caricamento dinamico e consultazione catalogo.
- Aggiornato `src/main.rs`:
  - `apply_provider_cooldown` esteso con classificazione semantica: cooldown dinamico preciso (invece di 60s fissi ciechi), 1h su quota esaurita.
  - Aggiunti endpoint `/catalog/free` (ispezione catalogo 100 provider con stato `is_active`) e `/catalog/activate/:id` (attivazione a caldo dinamica nel pool).
  - Collegata sanitizzazione RTK su percorsi OpenAI e Anthropic.
- **Verifica Empirica**:
  - `cargo test`: 13 test passati (0 falliti).
  - `cargo build --release`: build completata con successo.
  - Servizio systemd `siliceo-nexus.service` riavviato e attivo con soli 3.2 MB di RAM.
  - Test live `POST /v1/chat/completions` riuscito con successo (consegna risposta con token $0.00).
  - Test live RTK riuscito.
  - Test live `/catalog/activate/opencode` riuscito con attivazione a caldo a runtime.

### 🎁 Roster 100 Provider Free-Tier & Dashboard Interattiva
- **Bonifica Catalogo (`data/free_providers_catalog.json`)**:
  - Eliminati fake protocolli stdio (`auggie://`, `zcode://`) e sostituiti con veri endpoint HTTP cloud (`Together AI`, `Cloudflare Workers AI`, `GitHub Models Free Tier`).
  - Corretti i path duplicati su `/chat/completions` ed eliminati modelli obsoleti non più esistenti (es. `llama-3.3-70b-versatile` su Groq sostituito con i modelli verificati live `openai/gpt-oss-120b` e `qwen/qwen3.8-27b`).
  - Arricchiti tutti i 100 provider con campi: `signup_url` (link diretto alla console di registrazione/creazione chiavi), `free_tier_info` (guida dettagliata in italiano su quote, limiti e come accedere al tier gratuito) e `category`.
- **Evoluzione Dashboard Web (`src/dashboard.rs`)**:
  - Aggiunto il tab dedicato `🎁 Roster 100 Free Tier` con conteggi in tempo reale (Tutti: 100, Attivi nel Pool, Da Attivare, Zero-Auth).
  - Barra di ricerca unificata (cerca per nome, modello, categoria, tag, informazioni free tier) e filtri a pillola.
  - Card dinamiche per ogni provider con badge di categoria, badge di stato (`🟢 ATTIVO` vs `⚪ DA ATTIVARE`), box `💡 Come funziona il Free Tier`, variabile d'ambiente suggerita (`env: ...`) e link rapido `🌐 Registrati / Ottieni Chiave ↗`.
  - Modale interattivo `🚀 Attiva Provider Free-Tier`: apre console, mostra istruzioni, permette l'inserimento della chiave API e la selezione rapida del modello tramite chip, inviando la richiesta a `POST /catalog/activate/:id` per l'inserimento a caldo nel pool a runtime.
  - Riconoscimento intelligente dei provider attivi nel pool tramite match su ID, nome e prefissi (es. `groq-free-pool` agganciato a `Groq`).
