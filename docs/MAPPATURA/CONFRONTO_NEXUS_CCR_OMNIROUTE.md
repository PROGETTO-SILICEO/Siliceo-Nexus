# Confronto Nexus vs CCR vs OmniRoute — Matrice Gap

> Data: 2026-08-10/11 · Autore: Sempre · Stato: analisi statica completa dei tre progetti
> Fonti: rapporti file-per-file in `docs/MAPPATURA/RAPPORTO_*.md` e mappe graphify (`*_GRAPH_REPORT.md`)
> Obiettivo: capire cosa Nexus deve fare per diventare un gateway maturo, usando CCR e OmniRoute come riferimento

---

## 1. Identità dei tre progetti

| | **Nexus** | **CCR (claude-code-router)** | **OmniRoute** |
|---|---|---|---|
| Tipo | LLM gateway Rust, binario singolo | Proxy/rewriter per Claude Code (TS monorepo) | Unified AI router (TS, 291 provider) |
| Linguaggio | Rust | TypeScript | TypeScript |
| Dimensione core | ~3.4k righe / 8 file | 183 file core TS | 11.5k file (src/domain 24 file core) |
| Endpoint principale | `:8082/v1/chat/completions`, `/v1/messages` | proxy su porta del client | `:3001/v1/*` |
| Modello di esecuzione | Processo unico | Gateway "spesso" + core gateway in subprocess (`@the-next-ai/ai-gateway`) | Next.js + worker, con mitm/stream |
| Provider | 11 configurati, free | Presets + qualunque provider via capability | 291 provider |

---

## 2. Matrice funzionale (gap analysis)

Legenda: ✅ presente e funzionante · 🟡 parziale/fragile · ❌ assente · 🔵 non applicabile

| Funzionalità | Nexus | CCR | OmniRoute | Nota |
|---|---|---|---|---|
| **Routing per intent/tag** | ✅ (classify_intent + tag) | ✅ (policy engine, condition/model-prefix) | ✅ (tagRouter, categoria task) | CCR/OmniRoute più espressivi |
| **Fallback provider** | 🟡 cascata sequenziale per priorità | ✅ fallback chain protocolli | ✅ fallbackPolicy dichiarativa + excludeProviders | OmniRoute: fallback come **dato**, non codice |
| **Cooldown provider** | ❌ `cooldown_until` letto ma MAI scritto | ✅ credential pool con cooldown 401/403/429/5xx | ✅ lockoutPolicy + cooldown per errore | Nexus promette ma non implementa |
| **Multi-key round-robin** | 🟡 `timestamp%n`, senza stato | ✅ credential pool con peso+spillover | ✅ combo round-robin | Nexus: pseudo-random, non vero RR |
| **Streaming SSE** | ❌ JSON bloccante | ✅ completo (SSE adapter per protocollo) | ✅ streaming nativo | **CAUSA PRINCIPALE blocco Claude Code** |
| **Tool call conversion** | ❌ messaggi tool_use/tool_result buttati | ✅ formato standard interno + adapter | ✅ (domain models) | Nexus: `Message` senza tool_calls |
| **Formato standard interno** | ❌ conversione diretta per provider | ✅ standard a 3 livelli (protocollo/capability/core) | 🔵 (domain types propri) | CCR: multi-protocollo pulito |
| **Passthrough stesso protocollo** | ❌ | ✅ (RN: body intatto se stesso protocollo) | 🔵 | CCR evita conversione inutile |
| **Context archive (memoria post-compattazione)** | ❌ | ✅ snapshot SQLite + lineage + MCP replay | 🔵 (compressione, non archive) | **Pattern più avanzato di CCR** |
| **Compressione contesto** | ❌ | 🟡 (context archive side) | ✅ RTK+Caveman ladder adattiva | OmniRoute: ladder con floor |
| **Quota/rate limit** | 🟡 globale 120/min, bypassabile via /v1/messages | ✅ window counter per key+credential | ✅ quota engine fair-share work-conserving | OmniRoute: pre-check/post-track fail-open |
| **Budget/costo** | ❌ `estimated_cost_usd` sempre 0 | ✅ usage capture + billing sync | ✅ costRules con budget in proiezione | OmniRoute: `checkBudget(key, additionalCost)` |
| **Telemetria** | ❌ finta (uptime hardcoded, gpu finto) | ✅ route trace a hop con diff tipizzato | ✅ header `X-OmniRoute-*` + dashboard | Nexus: dashboard mostra numeri inventati |
| **Osservabilità decisioni routing** | ❌ | ✅ RequestRouteTrace (hop+diff+redazione) | 🟡 header di decisione | CCR: "perché è stato scelto X" |
| **Auth/SSRF** | ✅ SSRF, key masking, redazione | ✅ credenziali interne, auth token | ✅ middleware authz per classe rotta | OmniRoute: header trusted strippati |
| **CORS** | ❌ allow-any / assente | 🔵 | ✅ allowlist + fail-closed cookie | Nexus: non gestito correttamente |
| **Dashboard** | 🟡 valori finti, markup duplicato | 🟡 web management server | ✅ dashboard live | Nexus: da collegare a dati reali |
| **Sicurezza processi** | 🟡 beellama-switcher senza auth, path traversal | 🔵 | 🔵 | beellama: 0.0.0.0 senza auth |
| **API key auto-detection** | ✅ presets+rilevamento | ✅ login locale come provider | 🔵 | Nexus ok |
| **Catalog sync** | ✅ 24h OpenRouter+Google | 🔵 | 🔵 | Nexus ha 1141 modelli (fix etichette) |

---

## 3. Architetture chiave da cui Nexus deve imparare

### Da CCR
1. **Formato standard interno a 3 livelli**: client-protocollo → provider-capability → core-provider. Nexus oggi converte diretto Anthropic↔OpenAI; CCR definisce una catena di protocolli e sceglie il provider per capability.
2. **Gateway "spesso" + core "magro" in subprocess**: routing/rewrite/trace nel gateway, conversione protocollo in un core isolato. Nexus non ha bisogno di subprocess, ma il *concetto* di separare routing da conversione sì.
3. **Context archive**: snapshot immutabili SQLite con lineage + tool MCP che fa replay sul modello originale. È la memoria a lungo termine di un proxy — Nexus non ha nulla di simile e le sessioni lunghe di Claude Code ne soffrono.
4. **Route trace a hop con diff tipizzato**: osservabilità prodotta dai punti di mutazione, con budget e redazione. Prerequisito per "spiegare ogni decisione".
5. **Credential pool con cooldown/spillover/weight**: la versione *vera* di quello che Nexus promette nel README.

### Da OmniRoute
6. **Fallback dichiarativo + excludeProviders**: la catena di fallback è un dato (tabella), non un loop nel codice. `excludeProviders` impedisce loop. Nexus: la cascata sequenziale è codice, non dato.
7. **Assessment proattivo + tassonomia errori a 5 classi** (working/broken/rate_limited/timeout/auth_error): il gateway *sonda* i provider e classifica gli errori — auth_error non è ritentabile, rate_limited sì.
8. **Self-healing graduato**: de-pesa → rimuovi → emergency_replace con template. Nexus butta via i provider su 429 (o li lascia in cooldown per sempre).
9. **Quota engine fair-share work-conserving fail-open**: la quota non blocca mai su guasto infra, pre-check/post-track separati.
10. **Compressione adattiva a ladder**: motori ordinati per aggressività, si scala solo se non rientra, mai over-compress.

### Pattern trasversali
- **Telemetria come choke-point**: header `X-*` costruiti in un unico punto sanitizzato (OmniRoute) o trace con budget (CCR). Mai valori hardcoded (Nexus oggi).
- **Middleware authz per classe di rotta**: PUBLIC/CLIENT_API/MANAGEMENT con policy dedicate, header trusted strippati e re-innestati (OmniRoute).
- **Decision log inline**: i commenti con numeri di issue e decisioni (B16/B25) di OmniRoute sono una disciplina che Nexus deve adottare.

---

## 4. Le 10 priorità per Nexus (in ordine)

| # | Priorità | Intervento | Riferimento |
|---|---|---|---|
| 1 | **P0** | **Streaming SSE** su `/v1/messages` e `/v1/chat/completions` (senza, Claude Code va in timeout ~16s) | CCR SSE adapter / OmniRoute streaming nativo |
| 2 | **P0** | **Tool call conversion** end-to-end: `Message`/`Choice` con tool_calls/tool_use/tool_result | CCR formato standard interno |
| 3 | **P0** | **Cooldown provider reale**: scrivere `cooldown_until` su 401/403/429/5xx + backoff | CCR credential pool / OmniRoute lockoutPolicy |
| 4 | **P0** | **Multi-key round-robin vero** con stato, non `timestamp%n` | CCR credential pool |
| 5 | **P0** | **Telemetria reale**: rimuovere valori finti, collegare a usage_log e latenza misurata | OmniRoute choke-point / CCR route trace |
| 6 | **P1** | **Rate limit su `/v1/messages`** (oggi bypassabile) + quota per chiave | OmniRoute quota engine |
| 7 | **P1** | **Fallback dichiarativo** con exclude-providers, non cascata hardcoded | OmniRoute fallbackPolicy |
| 8 | **P1** | **Context archive** per sessioni lunghe (snapshot + replay) | CCR context archive |
| 9 | **P1** | **Assessment proattivo** + tassonomia errori (auth_error vs rate_limited) | OmniRoute assessor |
| 10 | **P1** | **Fix beellama-switcher**: streaming pass-through, readiness check, auth | OmniRoute/CCR |

---

## 5. Conclusione

Nexus è una base valida ma è a uno stadio **pre-maturo**: promette nel README funzionalità (cooldown, multi-key, tool conversion, telemetria) che nel codice non esistono o sono finte. CCR e OmniRoute confermano che il gap non è nella conversione protocollo (che Nexus fa in modo diretto ma corretto per 2 protocolli), ma in:

1. **Streaming** (requisito per Claude Code, non opzione)
2. **Resilienza** (cooldown, fallback dichiarativo, assessment, self-healing)
3. **Osservabilità** (telemetria reale, trace delle decisioni)
4. **Memoria di contesto** (context archive per sessioni lunghe)

La priorità assoluta: **rendere Nexus un gateway affidabile per Claude Code** (streaming + tool conversion + cooldown), poi aggiungere i pattern di maturità da CCR/OmniRoute.

---

## 6. Riferimenti

- `NEXUS_GRAPH_REPORT.md` (graphify: 179 nodi, 19 comunità)
- `CCR_GRAPH_REPORT.md` (graphify: 5919 nodi, 187 comunità)
- `OMNIROUTE_GRAPH_REPORT.md` (graphify: 302 nodi, 17 comunità — core domain)
- `RAPPORTO_NEXUS_FILE_PER_FILE.md` (321 righe)
- `RAPPORTO_CCR_FILE_PER_FILE.md` (448 righe)
- `RAPPORTO_OMNIROUTE_FILE_PER_FILE.md` (396 righe)
