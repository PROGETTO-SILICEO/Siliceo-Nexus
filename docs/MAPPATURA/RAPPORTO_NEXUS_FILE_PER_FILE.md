# Rapport File-per-File — Siliceo-Nexus v0.1.0 (pre-fix, commit `e0cba59`)

> **Data analisi:** 2026-08-10 — **Analista:** Sempre (architetta SW)
> **Oggetto:** analisi statica completa, senza modifiche al codice.
> **Contratto di riferimento:** `README.md` (feature promesse) confrontato con l'implementazione reale.
> **Stato del tree:** 3 file modificati non committati (`dashboard.rs`, `db.rs`, `main.rs`) + `docs/` e `graphify-out/` untracked. Versione binario: `0.1.0`.

**Codice analizzato: 3.353 righe in 8 file:**

| File | Righe | Ruolo |
|---|---|---|
| `src/main.rs` | 1024 | Router HTTP, handler, sicurezza, rate limit |
| `src/dashboard.rs` | 980 | SPA HTML/CSS/JS embedded (server-rendered string) |
| `src/db.rs` | 272 | SQLite, schema, seed, masking, permessi |
| `src/adapters.rs` | 281 | Dispatch verso provider (OpenAI/Anthropic/Gemini) |
| `src/catalog.rs` | 184 | Sync cataloghi OpenRouter + Google AI Studio |
| `src/router.rs` | 121 | Intent classifier + selezione provider |
| `src/types.rs` | 102 | Tipi di dominio |
| `beellama-switcher/src/main.rs` | 227 | Micro-daemon hot-swap GPU (nodo RTX 2070) |

---

## 1. `src/main.rs` (1024 righe)

### Cosa fa oggi
- **`main()`**: init tracing, DB SQLite WAL (`data/nexus.db`), carica i provider in memoria (`RwLock<Vec<Provider>>`), avvia `catalog::spawn_catalog_sync_loop`, espone ~19 route su `127.0.0.1:8082` (default).
- **Route principali:**
  - `/` → dashboard SPA (no auth)
  - `/v1/chat/completions` → endpoint OpenAI-compatibile
  - `/v1/messages`, `/messages`, `/v1/v1/messages` → endpoint Anthropic (alias duplicati come workaround)
  - `/v1/models`, `/models`, `/v1/v1/models` → list model (open, con alias Claude hardcoded)
  - `/providers` (GET/POST), `/providers/fetch_models`, `/providers/:id` (DELETE), `/providers/:id/test`, `/providers/:id/set_model`
  - `/catalog`, `/catalog/sync`
  - `/health`, `/stats` (telemetria)
- **`handle_chat_completions`** (riga 163): auth → rate limit → `classify_intent` → `select_eligible_providers` → **cascata di failover sequenziale** sui provider idonei, log per tentativo, `502` se esausto.
- **`handle_anthropic_messages`** (riga 269): converte `system` (string o array di blocchi) e messaggi (string o array di blocchi `text`) in `LLMRequest`. **`stream` forzato a `None`, `tools` forzato a `None`.** Risponde JSON bloccante in formato Anthropic, sempre `stop_reason: "end_turn"`.
- **`handle_v1_models`** (riga 218): modelli dal catalogo DB (LIMIT 2000) + 8 alias Claude hardcoded (`claude-sonnet-4-6`, `claude-3-5-sonnet-20241022`, ..., `sonnet`, `haiku`, `opus`).
- **Sicurezza**: `verify_token`/`verify_admin_auth`/`verify_api_auth` (Bearer; no-op se la env non e impostata), `is_safe_endpoint_url` (SSRF: blocca IP privati/speciali e HTTP non-trusted, check DNS, redirect disabilitati nel client), `redact_secrets` (masking chiavi nei log/errori), `enforce_inference_rate_limit` (finestra mobile 60s, default 120, env `NEXUS_MAX_REQUESTS_PER_MINUTE`), `ensure_network_auth_is_configured` (obbliga token se bind non-loopback).
- **`handle_fetch_models`** (riga 810): autodiscovery modelli da endpoint (gestisce Ollama `/api/tags`, dedup, stripping `/v1`, `/v1beta`, `/openai`), riuso della chiave salvata solo se `same_endpoint_origin`, inferenza del provider key dall'URL, `INSERT OR REPLACE` in catalogo.

### Cosa dovrebbe fare (divario vs README)
- **Streaming SSE**: il README promette "Full compatibility with Claude Code CLI, Claude Desktop, and Anthropic SDKs". Claude Code manda `stream: true` e **aspetta eventi SSE**; questo codice risponde JSON bloccante → timeout client (~16s). Mancano: parsing `stream: true`, flusso SSE, `delta.content`/`delta.reasoning`, `stop_reason` di streaming.
- **Tool call conversion**: il README dichiara "Tool call conversion: OpenAI `tool_calls` → Anthropic `tool_use` content blocks". **In questa versione non esiste**: `handle_anthropic_messages` scarta `tools`, estrae solo blocchi `text` e butta i blocchi `tool_use`/`tool_result` della storia del client.
- **Telemetria vera**: `/stats` promette "Real-time GPU load, system RAM utilization, latency sparkline". Il load GPU e **inventato**, l'uptime e una stringa finta.
- **Cooldown provider su 429**: README promette "Auto-cooldown on 429 errors". Nessun codice scrive `cooldown_until` (vedi router: la colonna e solo letta).
- **Rate limit per provider**: `tpm_limit`/`rpm_limit` esistono nel DB ma non sono mai applicati; esiste solo il limite globale.

### Punti deboli / bug / rischi
1. **`TOTAL_REQUESTS` e `LAST_LATENCY_MS` (righe 118-119) sono statici e MAI aggiornati**: partono da `14` e restano `14` per sempre → dashboard mostra sempre "14 ms".
2. **`handle_stats` (riga 121) produce telemetria finta**: `gpu_utilization_pct = ((total_reqs % 25) + 40)` (riga 149), `uptime: "99.9%"` hardcoded (riga 145). Nessun client verso la 2070.
3. **`handle_anthropic_messages` NON chiama `enforce_inference_rate_limit`** (a differenza di chat completions e models, righe 169 e 223) → **bypass del rate limit** via `/v1/messages`.
4. **Alias Claude hardcoded** in `handle_v1_models` e route duplicate `/v1/v1/*`: workaround, non architettura.
5. `handle_test_provider` ritorna l'errore raw (`e.to_string()`, riga 747) senza `redact_secrets` → leak parziale possibile (basso rischio: gli errori adapter non includono la URL, che per Gemini contiene `?key=`).
6. `handle_fetch_models`: nessun limite al numero di modelli inseriti; `INSERT OR REPLACE` con `context_length=131072` fisso e costi 0 → modelli default "free".
7. **`is_safe_endpoint_url`**: DNS lookup separato dall'uso reale → rischio TOCTOU/DNS-rebinding (mitigato parzialmente dai redirect disabilitati).
8. `handle_anthropic_messages` risponde sempre `stop_reason: "end_turn"` anche per richieste con tools → incoerente.
9. `verify_api_auth` e no-op senza env: accettabile in locale, ma su rete e coperto solo da `ensure_network_auth_is_configured`.
10. **Nessuna scrittura su `usage_log`** (tabella creata in db.rs ma mai popolata): la telemetria di utilizzo/costo promessa non esiste.

### Priorita di intervento
| Livello | Voce |
|---|---|
| **P0** | Streaming SSE su `/v1/messages` e `/v1/chat/completions` (Claude Code va in timeout senza) |
| **P0** | Tool call conversion assente (viola il contratto README) |
| **P0** | Telemetria finta in `/stats` + contatori mai aggiornati |
| **P1** | Rate limit non applicato a `/v1/messages` |
| **P1** | Errori test provider non redattati |
| **P1** | Scrittura `usage_log` per telemetria reale |
| **P2** | Alias Claude hardcoded → derivare da catalogo/config |

---

## 2. `src/types.rs` (102 righe)

### Cosa fa oggi
Definisce il contratto dati: `Provider`, `ProviderInput`, `LLMRequest`, `Message`, `Choice`, `LLMResponse`, `UsageInfo`, `CatalogItem`, con default serde per i campi opzionali.

### Cosa dovrebbe fare / limiti strutturali
- **`Message` ha solo `role` + `content: String`** → impossibile rappresentare `tool_calls` (OpenAI), `tool_use`/`tool_result` (Anthropic), content blocks multimodali, thinking blocks. **Questa è la radice strutturale che rende impossibile la "Tool call conversion" promessa.** Serve un tipo content astratto (enum di block types: text, tool_use, tool_result, image, thinking).
- **`Choice.message` è un `Message`** → la risposta OpenAI non può trasportare `tool_calls`; i client agentici non vedono mai le chiamate tool.
- **`LLMRequest.tools` è `Option<serde_json::Value>`** → nessuna validazione/typing; il forward a provider diversi è fragile.
- **`ProviderInput` non ha `id`** → `POST /providers` non può aggiornare per id; il frontend compensa con DELETE+POST (vedi dashboard, rischio perdita dati).
- `estimated_cost_usd` esiste ma è **sempre 0.0** in ogni adapter → la "cost awareness" promessa non esiste.
- `CatalogItem` non è usato dal codice attivo (il catalogo è gestito con `serde_json::Value` in main.rs).

### Punti deboli / bug
- Nessun campo per `stream_options`, `n`, `stop`, `frequency_penalty` → il gateway non può essere trasparente rispetto al protocollo OpenAI/Anthropic.
- `finish_reason` è una stringa libera ma in pratica sempre `"stop"`.

### Priorità di intervento
| Livello | Voce |
|---|---|
| **P0** | Redesign `Message`/`Choice` per content blocks + `tool_calls` (prerequisito tool conversion) |
| **P1** | Aggiungere `id` a `ProviderInput` per update by-id |
| **P1** | Calcolo costo stimato reale (catalogo × usage) |
| **P2** | Campi protocollo mancanti (`n`, `stop`, penalties) |

---

## 3. `src/adapters.rs` (281 righe)

### Cosa fa oggi
- **`dispatch_request`** (riga 5): dispatch per `auth_type`: `"anthropic"` → `try_anthropic`, `"gemini-query"` → `try_gemini_native`, altrimenti `try_openai_compatible`.
- **`pick_api_key`** (riga 18): pseudo round-robin = `timestamp_subsec_nanos() % len(keys)` dopo split su `,`/`;`. **Nessuno stato di rotazione, nessuna gestione 429.**
- **`try_openai_compatible`** (riga 40): body con `max_tokens` default 4096, `temperature` default 0.7, forward `tools`; URL costruita da `base_url`; chiave per nome provider (`gemini-free-tier`/`groq-free-pool` altrimenti env `OPENROUTER_API_KEY`); Gemini via query `?key=`; auth `bearer`/`api-key`; timeout 45s; estrae solo `content` e `usage`.
- **`try_gemini_native`** (riga 135): URL hardcoded `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions?key=...`; **non usa `provider.base_url`**. Con i seed attuali (auth_type `bearer`) **questo ramo è morto**: Gemini passa da `try_openai_compatible`. Codice duplicato.
- **`try_anthropic`** (riga 197): estrae il system prompt dai messaggi, converte il resto in `messages` string, header `x-api-key` + `anthropic-version: 2023-06-01`. **Nessun timeout esplicito.** Estrae solo `content[0].text` → ignora `tool_use`, `thinking`, `usage` Anthropic.

### Cosa dovrebbe fare (vs README)
- **"Multi-Key Round-Robin Rotation ... Automatically rotates keys and handles `429 Too Many Requests` without interrupting sessions"**: serve stato per chiave (indice rotante per provider, contatore fallimenti), skip della chiave in 429, marcatura `cooldown_until`. Oggi c'è solo `timestamp % n` senza memoria né gestione errori.
- **"Auto-cooldown on 429 errors"**: nessun ramo rileva 429 e scrive `cooldown_until`.
- **"Tool call conversion"**: gli adapter devono (a) parsare `tool_calls` nella risposta OpenAI e (b) mappare `tools`/`tool_use`/`tool_result` nei due protocolli.
- **Streaming**: il campo `stream` non è nemmeno forwardato nel body → nessun supporto SSE.
- **Telemetria per richiesta**: nessun ritorno di latenza/stato 429 per popolare `/stats` o `usage_log`.

### Punti deboli / bug / rischi
1. `pick_api_key` non garantisce rotazione equa né isolamento tra richieste concorrenti; una chiave morta nel pool causa fallimenti deterministici quando il nanosecondo cade su di essa.
2. **`try_anthropic` senza timeout** → può appendere per sempre.
3. `finish_reason` sempre `"stop"`, `object` sempre `"chat.completion"` → incoerente con risposte troncate/errori.
4. `estimated_cost_usd` sempre 0 (vedi types).
5. `try_gemini_native` duplicato e morto (dipende dall'auth_type del seed, non dall'endpoint).
6. L'errore provider arriva grezzo a `main` (che lo redige solo su alcune route).
7. `max_tokens` fisso 4096 → risposte reasoning lunghe (nemotron/Qwen reasoning) tagliate; dai round successivi è noto che serviva alzarlo.
8. Nessun handling dello schema OpenAI `reasoning_content`/`delta.reasoning` (nemotron emette il testo lì).

### Priorità di intervento
| Livello | Voce |
|---|---|
| **P0** | Implementare vero multi-key rotation + handling 429 + scrittura `cooldown_until` |
| **P1** | Streaming SSE negli adapter |
| **P1** | Parsing `tool_calls`/`tool_use` nei due protocolli |
| **P1** | Timeout esplicito in `try_anthropic` |
| **P1** | Forward di `reasoning_content`/`delta.reasoning` per modelli reasoning |
| **P2** | Eliminare `try_gemini_native` (o usare `provider.base_url`) |
| **P2** | `max_tokens` configurabile per provider |

---

## 4. `src/router.rs` (121 righe)

### Cosa fa oggi
- **`IntentTag`**: `Chitchat / Coding / Reasoning / ToolCall` con `as_str()`.
- **`classify_intent`** (riga 26, euristico, <1ms): `tools.is_some()` → `ToolCall`; parole chiave codice (```, `fn `, `def `, `return `, `traceback`...) → `Coding`; parole "architettura/pianifica/valuta/perché" → `Reasoning`; <80 char o saluti ("ciao", "chi sei", "presenza") → `Chitchat`; default **`Reasoning`**.
- **`select_eligible_providers`** (riga 71): filtra `enabled`, salta provider in cooldown (legge `cooldown_until` rfc3339), filtra `tool_supported` se `requires_tools`, poi **ri-aggiunge TUTTI i provider abilitati come fallback** (riga 99-103).
- **`select_provider`** (riga 109): primo degli eligible. **Non è usato da nessun handler** (solo `select_eligible_providers` è chiamato).

### Cosa dovrebbe fare (vs README)
- **"Tag & Capability-Based Routing. Zero hardcoded model names ... capabilities (coding, fast, local, tool_supported)"**: il fallback che ri-aggiunge *tutti* i provider abilitati svuota il routing per tag: l'intento ordina ma non esclude. La colonna `capabilities` del catalogo non è mai consultata.
- **tpm/rpm**: `tpm_limit`/`rpm_limit` mai applicati → la promessa "cost constraints / rate awareness" non c'è.
- **Cooldown**: la lettura di `cooldown_until` esiste ma nulla la scrive (vedi main/adapters) → è codice morto.

### Punti deboli / bug
- Classifier euristico grossolano: parole comuni ("return ", "let ", "code") causano falsi positivi `Coding`; default `Reasoning` per qualunque messaggio lungo.
- Il doppio loop (tag + fallback totale) rende il failover non determinista rispetto all'intento: un provider chitchat-only può rispondere a richieste di coding se i tag-coding falliscono.
- `select_provider` duplica la logica e non è utilizzato → codice morto.

### Priorità di intervento
| Livello | Voce |
|---|---|
| **P1** | Rimuovere il fallback "tutti i provider" o renderlo esplicito a priorità (es. solo tier `local`) |
| **P1** | Applicare `tpm_limit`/`rpm_limit` per provider |
| **P2** | Migliorare il classifier (weighted keywords, token-based) o documentare i limiti |
| **P2** | Rimuovere `select_provider` o usarlo nel flusso |

---

## 5. `src/catalog.rs` (184 righe)

### Cosa fa oggi
- **`sync_openrouter_catalog`** (riga 39): GET `https://openrouter.ai/api/v1/models` (timeout 20s), parse, `INSERT OR REPLACE` in transazione con prezzi `* 1M`, `is_free` = entrambi 0.0, `context_length` default 32768, `capabilities` sempre `["text","chat"]`.
- **`sync_google_catalog`** (riga 99): cerca `GEMINI_API_KEY` (env o DB: name `gemini-free-tier` o base_url con generativelanguage), GET `.../v1beta/models?key=`, `is_free=1`, `context_length` default 1048576. Se manca la chiave → `Ok(0)` silenzioso.
- **`sync_all_catalogs`** (riga 168): esegue entrambi, `unwrap_or(0)` su errore.
- **`spawn_catalog_sync_loop`** (riga 175): task background che fa sync all'avvio e ogni 86400s (24h).

### Cosa dovrebbe fare (vs README)
- **"Live 450+ Catalog Sync (24h)"** → ok, ma serve: backoff su errore (oggi se il sync fallisce si ritenta comunque dopo 24h), versioning/ETag, e sincronizzazione di altri provider (groq, mistral, cerebras...) promessi dai preset.
- **"Tag & Capability-Based Routing"**: la colonna `capabilities` è sempre hardcoded `["text","chat"]` → il routing per capabilities non ha dati reali.

### Punti deboli / bug
1. **Prezzi non parsabili → 0.0 → modello marcato FREE** (riga 59-68): un modello a pagamento con pricing malformato entra nel free pool. Rischio costi.
2. Google: `is_free=1` per **tutti** i modelli AI Studio, anche a pagamento (riga 151).
3. Nessuna pulizia dei modelli rimossi dal provider (INSERT OR REPLACE non cancella gli orfani).
4. `capabilities` fisse e mai aggiornate da OpenRouter (che espone `supported_parameters`).
5. Sync Google legge la chiave dal DB in chiaro: accettabile internamente, ma va notato che la chiave viaggia in query string (riga 122).
6. Loop background senza gestione panico: se il task muore, il catalogo non si aggiorna più.

### Priorità di intervento
| Livello | Voce |
|---|---|
| **P1** | Prezzi: fallire con warn invece di marcare free; marcare is_free da sorgente OpenRouter (campo `free`) |
| **P1** | Popolare `capabilities` reali da OpenRouter/Google |
| **P2** | Backoff/retry + sync per altri provider preset |
| **P2** | Pulizia orfani + ETag |

---

## 6. `src/db.rs` (272 righe)

### Cosa fa oggi
- **`mask_api_key`** (riga 7): masking `xxxx...yyyy` (4+4), gestisce chiavi già mascherate.
- **`set_secure_file_permissions`** (riga 23): `0o600` su Unix; su Windows solo log (ACL non implementate).
- **`set_secure_database_permissions`**: applica a `-wal`, `-shm`, `-journal`.
- **`init_db`** (riga 52): SQLite `create_if_missing`, WAL, `busy_timeout` 5s, pool 10 conn. Crea `providers`, `models_catalog`, `usage_log`. Chiama `seed_default_providers`.
- **`seed_default_providers`** (riga 131): 3 provider iniziali se la tabella è vuota:
  - `beellama-tailscale-2070` (priority 1, local, `http://100.98.20.76:8080`, model `Qwen3.5-4B-Q6_K.gguf`)
  - `openrouter-free-pool` (priority 2, free, **model `openrouter/free-models` — inesistente**)
  - `gemini-free-tier` (priority 3, free, model `gemini-2.5-flash`)
- **`insert_provider_db`** (riga 191): `INSERT OR REPLACE`, **preserva la chiave esistente se il payload è mascherato/vuoto/assente** (fix noto dal round precedente). **Non preserva `cooldown_until`**.
- **`load_all_providers`** (riga 239): SELECT ordinata per priority, parse tags JSON.

### Cosa dovrebbe fare (vs README)
- **"SQLite WAL ... high-throughput concurrent access"** → ok. Manca però la **persistenza della telemetria**: `usage_log` è creata ma mai scritta (nessun INSERT in tutto il codice).
- **"Cross-Platform Storage Security"**: su Windows la parte ACL non è implementata (solo log).

### Punti deboli / bug
1. **`INSERT OR REPLACE` azzera `cooldown_until`** (la colonna non è nell'INSERT): una modifica al provider cancella lo stato di cooldown. Critico una volta implementato il cooldown.
2. **`INSERT OR REPLACE` cambia l'`id`** (DELETE+INSERT sotto il capo): gli id cambiano a ogni update via POST → riferimenti frontend/sessioni potenzialmente stale.
3. Seed `openrouter/free-models` **non è un modello reale**: il failover verso OpenRouter fallirà con 404/400 se non sovrascritto.
4. Seed hardcoded `100.98.20.76:8080` (IP Tailscale in codice) e porta beellama: va tenuto allineato con la configurazione reale (8091 interna / 8080 switcher).
5. Chiavi API in chiaro a riposo (mitigate da 0o600 ma non cifrate).
6. `usage_log` morta: nessun INSERT, nessun endpoint di lettura, nessun aggregato per dashboard.

### Priorità di intervento
| Livello | Voce |
|---|---|
| **P0** | Fix seed `openrouter/free-models` → modello reale del free pool |
| **P1** | Popolare `usage_log` (per provider/model/tokens/cost/intent) + endpoint di lettura |
| **P1** | Preservare `cooldown_until` in `insert_provider_db` |
| **P2** | Update per id (evitare REPLACE che cambia id) |
| **P2** | ACL Windows reali |

---

## 7. `src/dashboard.rs` (980 righe)

### Cosa fa oggi
- **`render_dashboard()`**: restituisce una SPA monolitica (HTML+CSS+JS embedded) con:
  - Header con token admin (sessionStorage), search globale.
  - 4 stat-card con **valori statici hardcoded**: `17`, `450`, `17`, `99.8%`.
  - Tab: Provider Catalog, Model Gateway (catalogo), Live Telemetry.
  - Modal per registrare/modificare provider con **17 preset** (`PRESETS`) e auto-detect della chiave dal prefisso (`gsk_`, `AIzaSy`, `sk-ant-`, ...).
  - Azioni per card: Test, Cambio Modello (`promptChangeModel` → `POST /providers/:id/set_model`), Modifica, Elimina.
  - Catalogo con filtro per sorgente/free e limite di 500 righe.
  - Telemetria: `loadLiveStats()` poll `/stats` ogni 2s, sparkline con `sparklineHistory` **hardcoded** `[40,55,30,60,45,80,50,65]`.
- **`saveProvider`**: **la modifica è implementata come DELETE poi POST** (righe 707-709) → non transazionale.

### Cosa dovrebbe fare (vs README)
- **"Live Hardware & Gateway Telemetry ... real-time GPU load"**: la tab è a posto come UI ma mostra dati finti (vedi `/stats`). Deve leggere metrica reale o mostrare "N/A".
- **"Multi-Key Management: Pair multiple keys ... KEY_1, KEY_2, KEY_3"**: il campo chiave è un singolo input password; nessuna UI dedicata alle pool multiple (funziona solo incollando `KEY_1,KEY_2`).
- **"Dynamic Catalog Tabs (450+ models)"**: presente ma il backend LIMITA a 2000 e il JS a 500 righe → sorgenti con molti modelli invisibili (mitigazione dichiarata nel commento riga 877).
- **"1-Click Model Hot-Swapping"**: presente per il provider; ma il cambio modello su beellama richiede comunque lo switch del GGUF sul nodo (due step).

### Punti deboli / bug / rischi
1. **Edit = DELETE+POST**: se il POST fallisce (rete/errore), il provider originale è già cancellato → **perdita di dati**.
2. **Valori statici ingannevoli** (`17`, `450`, `99.8%`) mostrati prima del primo load e come label fisse nella telemetria ("Node: beellama-tailscale-2070", "RTX 2070 GPU" hardcoded nel pannello, righe 236/243).
3. Il token admin in `sessionStorage` è recuperabile via XSS: con HTML escaped correttamente il rischio è basso, ma va valutato (CSRF/lato browser).
4. Telemetria GPU hardcoded anche nel layout statico (riga 237: `45%`, barra al 45%).
5. Nessuna gestione errori visibile quando admin token assente: `loadProviders` fallisce silenziosamente (catch → console.error).

### Priorità di intervento
| Livello | Voce |
|---|---|
| **P1** | Fix edit: usare update by-id (richiede `id` in `ProviderInput`) invece di DELETE+POST |
| **P1** | Collegare telemetria a dati reali o mostrare "N/A" |
| **P2** | UI multi-key dedicata |
| **P2** | Rimuovere hardcode statistiche/GPU nel layout |

---

## 8. `beellama-switcher/src/main.rs` (227 righe)

### Cosa fa oggi
- **`main()`**: init tracing, `models_dir` (default `/home/alforiva/inference/models`), `beellama_bin` (llama-server), `internal_port` (default **8081**), listen `0.0.0.0:8080` (default). CORS `AllowOrigin::Any`, nessuna auth.
- **Route**: `/health`, `/v1/models` + `/models`, `/v1/switch_model` + `/switch_model`, `/v1/chat/completions` + `/v1/completions`.
- **`handle_list_models`** (riga 108): scansione dir per `.gguf`/`.bin`/`.tq`.
- **`handle_switch_model`** (riga 139): kill del processo attivo (se presente), spawn `llama-server --model <path> --port <internal> --host 127.0.0.1 -ngl 99 -c 8192`. **Risponde subito senza verificare che il server sia pronto.**
- **`handle_proxy_chat`** (riga 200): proxy verso `127.0.0.1:<internal>/v1/chat/completions`, forward di tutti gli header tranne `host`/`content-length`, **bufferizza l'intera risposta con `resp.bytes()`**.

### Cosa dovrebbe fare (vs README)
- **"Zero-GC Rust Micro-Daemon ... manages llama-server process hot-swapping on demand"**: ok concettualmente; manca: readiness check post-spawn, monitoraggio del processo (auto-restart su crash), cleanup degli zombie.
- **"Standalone 2MB micro-service"**: ok binary; ma **esposto su 0.0.0.0 senza auth** → chiunque nella rete/mesh può swappare il modello (DoS) o leggere la lista.
- **Proxy OpenAI-compatibile**: il README lo posiziona come endpoint del nodo GPU; oggi bufferizza tutto → **rompe lo streaming SSE** dei client (le risposte streaming restano in memoria finché il modello non ha finito).

### Punti deboli / bug / rischi
1. **`resp.bytes()` bufferizza l'intera risposta** (riga 222): streaming distrutto; per risposte molto lunghe rischio memoria/timeout.
2. **Path traversal**: `state.models_dir.join(model_name)` (riga 148) senza sanitizzazione di `../` → possibile puntare a file arbitrari esistenti (passato a llama-server con `--model`).
3. **Nessuna auth** su un servizio che controlla processi (switch model) → attacco da rete.
4. **Race condition**: switch risponde prima che llama-server ascolti → le prime richieste dopo lo switch falliscono.
5. **Processo figlio non monitorato**: se llama-server crasha, `active_process` resta un `Child` morto; il prossimo `kill()` può fallire silenziosamente; nessun restart.
6. Default `internal_port=8081` in conflitto con l'ambiente (da ledger: 8081 è di alforiva; la porta interna corretta è 8091).
7. CORS `Any` non necessario per un endpoint interno (lo usa Nexus lato server).

### Priorità di intervento
| Livello | Voce |
|---|---|
| **P1** | Streaming pass-through (non bufferizzare) |
| **P1** | Readiness check + monitoraggio/restart del processo |
| **P1** | Sanitizzazione del path model + auth token |
| **P2** | Fix default porta interna (8091) |
| **P2** | CORS ristretto |

---

## 9. Sintesi esecutiva — Le 10 cose più importanti da sistemare

Ordinate per impatto/urgenza (P0 → P2):

| # | Priorità | Area | Intervento |
|---|---|---|---|
| 1 | **P0** | `main.rs` / `adapters.rs` | **Streaming SSE** su `/v1/messages` e `/v1/chat/completions` (Claude Code va in timeout ~16s senza; è il requisito che ha bloccato i round 3-6 reali) |
| 2 | **P0** | `types.rs` + `main.rs` + `adapters.rs` | **Tool call conversion** end-to-end: blocchi `tool_use`/`tool_result`/`tool_calls` (oggi tutto è buttato); prerequisito = redesign `Message`/`Choice` |
| 3 | **P0** | `main.rs` `handle_stats` | **Telemetria reale**: rimuovere `gpu_utilization_pct` finto, `uptime "99.9%"`, contatori mai aggiornati; collegare a `usage_log` e latenza misurata |
| 4 | **P0** | `adapters.rs` `pick_api_key` + `router.rs` | **Multi-key round-robin vero + handling 429 + scrittura `cooldown_until`** (README le promette; oggi non esiste) |
| 5 | **P0** | `db.rs` seed | **Fix seed `openrouter/free-models`** (modello inesistente → failover OpenRouter rotto di default) |
| 6 | **P1** | `main.rs` | **Rate limit su `/v1/messages`** (bypass oggi via endpoint Anthropic) |
| 7 | **P1** | `db.rs` / `main.rs` | **Popolare `usage_log`** e usarla per costo stimato e statistiche dashboard (oggi tabella morta, `estimated_cost_usd` sempre 0) |
| 8 | **P1** | `dashboard.rs` + `types.rs` | **Edit provider transazionale**: update by-id (serve `id` in `ProviderInput`) invece di DELETE+POST; preservare `cooldown_until` in `INSERT OR REPLACE` |
| 9 | **P1** | `beellama-switcher` | **Proxy streaming pass-through + readiness check + auth** (oggi bufferizza tutto, risponde prima che llama-server ascolti, nessuna auth su 0.0.0.0) |
| 10 | **P1** | `adapters.rs` | **Timeout esplicito in `try_anthropic`** + forward `reasoning_content`/`delta.reasoning` per modelli reasoning |

**Menzione d'onore (P2, da fare appena possibile):** sanitizzazione path model in beellama-switcher; eliminare `try_gemini_native` duplicato/morto; `max_tokens` configurabile; backoff del catalog sync; pulizia orfani catalogo; ACL Windows reali.
