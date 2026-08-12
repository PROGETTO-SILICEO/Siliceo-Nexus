# RAPPORTO Claude Code Router (CCR) — analisi file per file del core

**Analisi**: Sempre, architetta SW — 2026-08-10
**Ogg etto**: `/tmp/claude-code-router/packages/core/src/` (183 file TypeScript)
**Metodo**: lettura statica (node_modules NON installati, nessuna esecuzione)
**Obiettivo**: capire come è costruito un proxy/gateway LLM che inoltra Claude Code ad altri provider, per estrarre architetture e pattern replicabili in Nexus.

---

## 0. Panoramica generale dell'architettura

CCR è un **router/gateway LLM a doppio strato**, costruito come applicazione Electron/desktop con un **core TypeScript puro**:

```
Claude Code / Codex / Grok / Kimi / OpenCode / Zcode / pi / kilo
        │  (HTTP, protocollo nativo del client)
        ▼
  ┌───────────────────────────┐      ┌────────────────────────────┐
  │  Gateway CCR (frontend)   │      │  Core Gateway (child proc) │
  │  porta config.gateway.port│◄────►│  porta gateway.corePort    │
  │  - auth API key + limiti  │ IPC/  │  - providers, MCP, media, │
  │  - pipeline richiesta     │HTTP   │    virtual model, plugins │
  │  - routing (regole)       │       │  - basato su @next-ai/   │
  │  - context archive        │       │    ai-gateway (bundlato)  │
  └───────────────────────────┘      └────────────────────────────┘
```

**Le due innovazioni architetturali principali rispetto a un semplice reverse proxy:**

1. **Gateway CCR "spesso"** (questo package): fa *routing semantico* (regole + policy engine + script worker), *rewrite* JSON path-based di body/header, *protocol adaptation* (traduce il protocollo del client in quello del provider), *context archive* (salva gli snapshot di compattazione e li rende queryable via MCP), *fallback multi-modello/multi-credential*, *osservabilità totale* (route trace), *limit/rate limiting per API key*.

2. **Core Gateway "magro" in subprocess**: invece di riscrivere un provider LLM gateway da zero, CCR **incapsula** `@the-next-ai/ai-gateway` (progetto esterno, package candidati `["@the-next-ai/ai-gateway", "gateway"]`) dentro un child process Node, gli inietta la configurazione via IPC + virtual config file, e ci mette davanti il proprio strato di routing. Questo è il pattern "gateway-bootstrap" descritto sotto.

L'architettura è a **confini netti** (vedi commenti ricorrenti `Extracted from gateway/service.ts. Keep this module focused on its named gateway boundary.`): file grandi storici sono stati estratti in moduli piccoli e focalizzati. Ogni directory è un confine, non un'accozzaglia.

---

## 1. `contracts/app.ts` (2472 righe) — IL contratto del sistema

**Percorso**: `packages/core/src/contracts/app.ts`
**Scopo**: tutti i tipi condivisi dell'intero sistema, centralizzati in un unico contratto.

**Cosa fa**: non contiene quasi logica (solo poche funzioni pure di utilità). È la fonte di verità dei tipi: se un modulo deve scambiare dati con un altro, usa questi tipi. Copre:

- `AppConfig` (riga 1749): la configurazione radice, un unico oggetto monolitico con TUTTE le sezioni: `Providers`, `Router`, `profile`, `proxy`, `gateway`, `contextArchive`, `plugins`, `APIKEYS`, `virtualModelProfiles`, `toolHub`, `mediaTools`, `observability`, `overviewWidgets` (dashboard), `trayWidgets` (UI desktop), `botGateway`.
- `GatewayProviderConfig` + `ProviderCredentialConfig` + `ProviderAccountConfig`: modello dati dei provider (vedi sezione providers).
- `GatewayProviderProtocol`: `openai_responses | openai_chat_completions | anthropic_messages | gemini_generate_content | gemini_interactions` + protocolli media (`openai_image_generations`, `openai_video_generations`, `xai_video_generations`). **Questo è il concetto chiave: il sistema ragiona a livello di protocollo, non di provider.**
- `RouterRule` / `RouterFallbackConfig` / `RouterRuleScript`: il DSL di routing (regole `condition`/`model-prefix`/`script`; fallback `off`/`retry`/`model-chain`; rewrite JSON-path).
- `VirtualModelProfileConfig`: "virtual models" = modelli fittizi (es. `Fusion/...`) che materializzano tool extra (web search, vision, media) aggiunti al body.
- `GatewayPluginConfig` + permessi (`GatewayPluginPermission`: `trusted-code`, `gateway-routes`, `proxy-routes`, `http-backends`, `sqlite-store`, ...) e superfici (`apps`/`gateway`/`provider`).
- `RequestLogEntry` + `RequestRouteTrace` (gli "hop" della traccia, vedi osservabilità).
- `UsageStatsSnapshot`, `AgentAnalysis*`: tutte le strutture dati per dashboard di utilizzo e analisi per-agente.
- `AppUpdateStatus`, `ProxyStatus`, `GatewayStatus`, `BotGateway*`: stato runtime esposto alla UI.
- Costanti nominali: nomi tool MCP built-in (`ccr-fusion-builtins`, `ccr-media-tools`, `ccr-context-archive`), limiti script (API v1, max 64KB source, timeout 2s default / 30s max).

**Architettura**: tipi + poche funzioni pure (es. `hasAvailableGatewayModels`, `availableGatewayModelIds`, `enforceSingleEnabledGlobalProfilePerAgent`). Nessuna dipendenza runtime: un contratto non importa nulla.

**Cosa dovrebbe fare / forza**: è l'esempio perfetto di **"contracts as the single source of truth"**. Per Nexus: definire un unico modulo di contratti puro (zero import), perché ogni modulo (config, routing, gateway, observability) possa importarlo senza creare cicli. Le funzioni pure dentro i contracts sono quelle che descrivono *invarianti* del dominio (es. "un modello gateway è visibile se...").

---

## 2. `config/` — Caricamento, validazione e persistenza della configurazione

### 2.1 `config/config.ts` (4203 righe)
**Scopo**: parse/merge/normalizzazione della config.

**Flusso principale**:
1. `loadRawAppConfig()` → sorgente gerarchica: **sqlite** (persistito) → **legacy JSON** (file, con `archiveLegacyJsonConfigFiles` per archiviare il legacy) → **default**.
2. `interpolateRawAppConfigEnvVars` → sostituzione `$VAR` nel config.
3. `pickConfig(value)` → **whitelist per sezione**: ogni sottosezione ha il proprio parser `parse*` (parseProviders, parseRouter, parseAgent, parseProxy, parseObservability, parseMediaTools, parseToolHub, parseProfile, parseBotGateway, parseGatewayPlugins, parseOverviewWidgets, parseTrayWidgets...). Ogni parser **valida, clamp, e ignora silenziosamente i campi sconosciuti**. Valori non validi → campo non presente (undefined), il default prevale.
4. Merge `DEFAULT_CONFIG` + picked, con regole di defaulting (es. `corePort = nextPort(port)`, `coreHost` forzato a `127.0.0.1`, `APIKEY` generata).
5. Sanitizzazione per disco: `sanitizeConfigForDisk` **azzera APIKEY/APIKEYS** (le chiavi vanno in tabella separata), forza coreHost loopback, strippa campi interni dai profili codex.
6. Scrittura con **coda serializzata** `enqueueAppConfigWrite` (una promise che incatena le scritture → niente race).

**Sicurezza**: `assertProviderApiKeysAreSafe` impedisce di salvare una API key che verrebbe inoltrata a un endpoint non di fiducia (controlla i preset e i connettori account). È una "security fail-closed" a livello config.

**Particolarità**: migrazione di plugin noti (`migrateKnownGatewayPluginConfigs` con defaults per claude-design/claude-ship/cursor-proxy), normalizzazione preset (es. NVIDIA: forza solo `openai_chat_completions`), sincronizzazione profili legacy ↔ profili nuovi.

### 2.2 `config/config-repository.ts`, `config/default-config.ts`, `config/constants.ts`, `config/onboarding-state.ts`
- **config-repository**: astrazione persistenza (load/save `AppConfig`, `ApiKeys`, snapshot) — verosimilmente via sqlite con fallback. Espone `loadPersistedApiKeys`, `replacePersistedConfigSnapshot`, ecc.
- **default-config**: `createDefaultAppConfig(...)` con tutti i default (porta, host, observer, MCP...).
- **constants**: `CONFIGDIR` (dir config), `CONTEXT_ARCHIVE_DB_FILE`, `LEGACY_*_CONFIG_FILE`.
- **onboarding-state**: stato del primo avvio (key generata).

**Forza**: il pattern *parse-whitelist-per-sezione + default + merge + sanitize-on-disk* è robustissimo: la config può contenere campi vecchi/sconosciuti e non si rompe; i segreti non toccano mai il disco nella config. Per Nexus: stessa triade "raw → normalizzato → sanitizzato", con coda di scrittura.

---

## 3. `routing/` — Il cuore decisionale

Questa è la directory più importante dal punto di vista "replicare CCR".

### 3.1 `protocol-endpoints.ts` (32 righe)
Mapping **path HTTP → protocollo**: `/v1/messages` → `anthropic_messages`, `/chat/completions` → `openai_chat_completions`, `/responses` → `openai_responses`, regex Gemini `/v1(beta)?/models/...:generateContent` e `/interactions`. `shouldApplyGatewayRouting` decide se una richiesta POST deve passare dal router. **Tutto il resto del sistema si aggancia a questo discriminator.**

### 3.2 `protocol-adapter.ts`
Traduce la *rappresentazione* del model tra i protocolli: `adaptRouteRequestBody` (estrapola/riposiziona il campo `model`, es. per Gemini il model sta nel path), `restoreRouteRequestBody`, `rewriteRouteModelInUrl` (per protocolli model-in-path), `routeModelFromPath`.

### 3.3 `model-registry.ts` — il registry dei modelli
`ModelRegistry` (classe) risolve **un selettore di modello → un riferimento a modello** (`RouteModelRef`: `{kind: "provider"|"gateway", provider, model, selector, canonicalSelector}`). Ordine di risoluzione:
1. provider esplicito (opzione `providerName`)
2. selettore `provider/model` parsato (`parseProviderModelSelector`)
3. modello gateway visibile (`availableGatewayModelIds`, include i virtual model)
4. match esatto sul nome modello → se univoco, ref; se ambiguo → undefined
5. match case-insensitive → se univoco.

Funzioni correlate: `normalizeRouteSelector` (accetta anche `provider,model` legacy con virgola), `providerRuntimeId` (id stabile con hash SHA-256 di nome+baseUrl, fallback sanitizzato — usato come **nome interno** nei header), `modelRegistryForConfig` con **WeakMap cache**.

### 3.4 `execution-plan.ts`
Trasforma `RouterFallbackConfig` + model primario in una **lista di tentativi** `RouteExecutionPlan`:
- `off` → 1 tentativo (il primario)
- `retry` → `retryCount+1` tentativi dello stesso model
- `model-chain` → sequenza `[primario, ...fallback.models]` deduplicata.

### 3.5 `rewrite.ts` (368 righe) — rewrite JSON-path sicuro
DSL di rewrite del tipo `request.body.model` o `request.header.X`. Operazioni: `set`, `delete`, `array-append`, `array-prepend`, `array-remove`, `array-replace`.
- **Compile-time**: `compileRouteRewrite` valida il path (`request.header[s].<nome>` o `request.body.<path>`), protegge gli header sensibili (`authorization`, `x-api-key`, `cookie`, `x-ccr-*`, `x-auth-*`), blocca path prototype-pollution (`__proto__`, `constructor`, `prototype`), converte i valori letterali (`true`, `null`, JSON, numeri).
- **Run-time**: `applyCompiledRouteRewrite` muta body/header e **restituisce un diff** (`RequestRouteTraceChange`) per la traccia.
- `effectiveBodyModelRewriteValue` / `effectiveTargetProviderName`: funzioni di proiezione che il config-compiler usa per capire "dove porta questa regola" senza eseguirla.

### 3.6 `config-compiler.ts` — compilazione delle regole
Trasforma le regole config in `CompiledRouterRule[]` **pronte all'esecuzione**: risolve i target model a compile-time, calcola i rewrites compilati, produce **diagnostica strutturata** (`RouteDiagnostic[]` con `code` tipizzato: `rule-rewrite-invalid`, `rule-model-not-configured`, `fallback-model-not-configured`, `script-api-unsupported`, ...). Se il fallback referenzia modelli non configurati, il fallback compilato viene **ridotto ai soli modelli validi** (fail-soft).

### 3.7 `policy-engine.ts` (28 righe) — policy chain
`RoutePolicyEngine` genera `RoutePolicy<TContext,TDecision>[]` con `evaluate()`; l'engine li valuta in ordine e **restituisce il primo match**. Pattern classico chain-of-responsibility, tipizzato sul contesto.

### 3.8 `route-script-*.ts` — script routing isolato
Regole `script` (JavaScript) eseguite **in worker threads** (`RouteScriptRuntime` → `RouteScriptWorkerSlot` → `Worker`), con:
- **circuit breaker per regola** (finestra di failure, open time),
- **coda con limite** (`queue-full`),
- **timeout**, pool di 1-4 worker, `validationCache` con hash,
- protocollo `route-script-worker-protocol.ts` (RPC request/response), `route-script-context.ts` (build input: body, headers, sessionId, tokenCount, toolNames, hasImage), `route-script-result.ts` (normalizza l'esito: matched, model, rewrites, fallback, diagnostics).

`route-script-worker.ts` è il file worker caricato da `Worker(file)` — **isolamento per file**, timeout e circuit breaker per regola = sicurezza rispetto a script utente.

### 3.9 `contracts.ts`
Tipi del routing: `RouteModelRef`, `RouteRequest`, `RouteExecutionPlan`, `RouteDiagnostic`, `RouteDecision`, `RouteSource` (`builtin|custom|default|profile|rule|subagent`).

**Forza complessiva del routing**: separazione netta tra (a) DSL dichiarativo, (b) compilazione con diagnostica, (c) engine runtime, (d) worker di esecuzione. Il design **compila la config a ogni (ri)avvio e produce diagnostica**, invece di validare a runtime. Nexus può copiare l'intero schema: `config-compiler → policy chain → execution-plan → executor`.

---

## 4. `providers/` — Topologia dei provider

### 4.1 `runtime-topology.ts` (501 righe) — il cuore del multi-provider
È il modulo che trasforma la config dei provider in una **struttura runtime utilizzabile**:

- `GatewayProviderCapability = { type, baseUrl, source: "detected"|"preset" }`: un provider può **esporre più protocolli su più base URL** (es. OpenRouter sia `openai_chat_completions` sia `openai_responses`).
- `providerProtocolForClientProtocol(provider, clientProtocol)`: data la richiesta del client (es. Anthropic), trova **con quale protocollo parlare al provider**, con **ordine di preferenza** (`providerProtocolPreferenceForClient`: per client `anthropic_messages` → `[anthropic_messages, openai_chat_completions, openai_responses, gemini...]`; per `openai_responses` → `[responses, chat, anthropic, gemini_interactions]`). **Questo è il pattern "protocol fallback chain"**: il client non deve sapere nulla del provider.
- `toCoreGatewayProviders`: espande `GatewayProviderConfig` (+ `credentials[]`) in `CoreGatewayProvider[]` **uno per capability e uno per credential** — il modello che viene iniettato nel core gateway.
- **Naming interno**: ogni istanza runtime ha un `providerRuntimeId` (o `provider::protocol` per capability, o `provider::protocol::cred:<slug>` per credential). Questi nomi interni viaggiano negli header `x-target-provider`, `x-target-providers`, `x-gateway-target-provider` e sono parsabili (`parseProviderCredentialInternalName`).
- `providerCredential*`: id/slug/priority/sort delle credential multiple di un provider.
- `sanitizeHeaderValue`: **header ByteString-safe** (i nomi provider in cinese/emoji crashano undici → normalizza a ASCII). Bug reale risolto in modo elegante.
- `inferProtocol`: heuristica su URL/transformer.
- `resolveResponseProviderProtocol`: ricostruisce il protocollo della risposta dagli header per l'osservabilità.

### 4.2 `presets/` — catalogo provider
`providerPresets: ProviderPreset[]` (26 preset: openai, anthropic, gemini, openrouter, nvidia, deepseek, kimi, zhipu, zai, minimax, mistral, moonshot, bailian, siliconflow, qiniu, fenno, infistar, runapi, teamorouter, unity2, code0, claudeapi...).

`ProviderPreset` (types.ts) = `{ id, name, aliases, endpoints: [{baseUrl, protocols[]}], officialApiKeyPatterns, websiteUrl, defaultModels, account }`.

`utils.ts`: matching base-URL → preset (`providerPresetMatchesBaseUrl`: confronta protocol+host+path prefix, tollera path "/"), identità fuzzy (`providerPresetIdentityMatchScore`), safety checks (`providerApiKeySafetyIssue*` — attualmente stub vuoti che ritornano `undefined`, cioè la safety è delegata altrove).

Esempio OpenRouter (openrouter/index.ts): account connector `http-json` che mappa `GET /api/v1/credits` → meter `balance` con espressioni JSON-path string (`limit: "$.data.total_credits"`). **I meter di bilancio sono dichiarati in config, non hardcoded nel codice**: i connettori account sono dati.

Esempio Anthropic: preset minimale con `defaultModels`, `officialApiKeyPatterns: ^sk-ant-`, un solo endpoint `anthropic_messages`.

### 4.3 Altri file providers/
- `url.ts`: normalizzazione base URL (aggiunge scheme, strip trailing slash).
- `credential-pool.ts`: gestisce lo **stato runtime delle credential** — cooldown (`cooldownMs=60s`) dopo 401/403/429/5xx, contatori di limit per finestra, e `recordProviderCredentialOutcome` chiamata dal gateway a ogni risposta.
- `probe.ts`: probe di connettività/modelli/protocolli su un base URL (usato dalla UI per l'aggiunta provider).
- `account-service.ts`, `manifest-service.ts`, `new-api.ts`, `oauth-plugin.ts`, `model-catalog.ts`, `icons.ts`: servizi satellite (bilanci conto, manifest deep-link, rilevamento provider new-api, plugin OAuth locale per provider, catalogo modelli).

**Forza**: la separazione **config (dati) / topologia (calcolo) / presets (catalogo)** è il cuore del multi-provider. Nexus deve replicare `GatewayProviderCapability` + `providerProtocolForClientProtocol` (chain di preferenza protocollo) + naming interno parsabile.

---

## 5. `gateway/` — Il runtime

### 5.1 Struttura
```
gateway/
├── application/gateway-service.ts   # ciclo di vita del server
├── auth/api-key-authorizer.ts       # auth + rate limit
├── core-runtime/                    # subprocess core gateway
│   ├── supervisor.ts                # spawn/supervisione child process
│   ├── gateway-bootstrap.ts         # entry IPC del child
│   ├── config-compiler.ts           # config CCR → config core gateway
│   └── upstream-header-sanitizer.ts, local-agent-auth-provider-hook.ts
├── http/                            # io, body, request-handler
├── internal/                        # shared, value, clock, collections
├── limits/window-limiter.ts         # window counters in-memory
├── request/pipeline.ts              # pipeline della richiesta (939 righe)
├── upstream/                        # executor.ts, retry-policy.ts
├── context-archive/                 # protocol.ts, store.ts + context-archive.ts
├── features/                        # compat layer per client specifici
├── claude-code-router-plugin.ts     # l'oggetto "router"
├── model-catalog.ts                 # catalogo dei modelli (metadati)
├── remote-control-service.ts        # endpoint di controllo remoto
└── service.ts                       # (estratto nei file sopra)
```

### 5.2 `request/pipeline.ts` (939 righe) — la pipeline del request
`GatewayRequestPipeline.proxyRequest()` è il **flusso principale**, una pipeline lineare di transformazioni sul body/header, ciascuna tracciata. Ordine reale delle fasi:

1. **Ingress**: leggi body, `requestId=uuid`, cattura `routeTrace.captureIngress()`.
2. **Normalizzazione header**: se autenticato via API key → strip header auth originali e inietta `x-auth-api-key-id`/`x-auth-sub`, `x-client-request-id=requestId`.
3. **Compat layer** (fase `compatibility`):
   - `prepareCursorOpenAICompatChatBody` (Cursor → OpenAI chat)
   - `prepareClaudeCodeDiscoveredModelRequest` (Claude Code → model discovery rewrite)
   - `prepareClaudeAppDiscoveredModelRequest` (Claude App)
   - `prepareCodexApplyPatchBridgeRequest`, `prepareCodexMultiAgentBridgeRequest`
   - `prepareCodexCompactCompatRequest`
4. **Limits**: `reserveApiKeyLimits` (429 se fuori quota).
5. **Serve risposte sintetiche**: `GET /models` (o `GET /v1/models`) → risponde direttamente con l'elenco modelli generato (`createGatewayModelsResponse`), senza inoltrare.
6. **Routing** (se `shouldApplyGatewayRouting`): `adaptRouteRequestBody` → `plugin.routeRequest(...)` → `restoreRouteRequestBody` → serializza. Gli esiti viaggiano negli header `x-ccr-route-reason`, `x-ccr-route-source`, `x-ccr-route-diagnostics`, `x-ccr-routed-model`. Il fallback effettivo può essere sostituito da quello deciso dal routing.
7. **Enrichment**:
   - hosted web search: `createHostedWebSearchProtocolContext` → se ci sono record recenti di browser search, **inietta i risultati nel body**; se il client li richiede ma non c'è integrazione → 503.
   - claude-code web search continuation: idem, su finestra 5min.
   - **context archive tool continuation** (`prepareContextArchiveToolContinuationRequest`): inietta il tool MCP di archive nel body se il client lo supporta.
   - **context archive** (`prepareContextArchiveRequest`): se c'è segnale di compattazione (header `x-ccr-context-compact`, `context_management.edits` con type `compact_*`, prompt auto-compact di Claude Code, path `/responses/compact`) → **snapshot del body** + append di un task di handoff (vedi §5.7). Può riscrivere il path upstream (`/responses/compact` → `/v1/responses`).
8. **Upstream**: `fetchUpstreamWithFallback` (vedi §5.4) con `AbortController` condiviso.
9. **Post-elaborazione risposta**:
   - `resolveContextArchiveToolContinuation` se il tool è stato invocato (replay).
   - `rewriteCapabilityResponseHeaders` (traduce i nomi interni provider → nomi pubblici nei header di risposta).
   - `finalizeContextArchiveRequest` (se successo) / `failContextArchiveRequest`.
   - **Stream pipeline**: build di una catena di Transform streams: `upstreamBody → codexApplyPatchBridge → codexMultiAgentBridge → hostedWebSearch → contextArchiveHandoff/codexCompact → anthropicModelRewrite`. Ogni anello è un `Transform` opzionale. `uniqueStreams` per evitare doppio attach di error handler.
   - **SSE error detector**: un detector sul flusso che distingue "client chiuso" da "errore upstream" anche in streaming.
   - **Body sampler** per il log (con max bytes e truncation), **usage capture** all'end del flusso.
10. **Gestione client disconnect**: `handleClientDisconnect` → abort upstream + log con status 499 (`clientClosedRequestStatusCode`).

Il tutto è punteggiato da `routeTrace?.capture({...})` con `phase` tipizzata (`ingress|compatibility|routing|capability|enrichment|planning|attempt|core|outcome`) — **ogni mutazione è osservabile e diagnoscibile** (vedi §8).

### 5.3 `upstream/executor.ts` (1169 righe) — l'esecutore
`fetchUpstreamWithFallback` è il motore del tentativo upstream:

1. **Planning**: `applyProviderCapabilityRouting` riscrive gli header target provider (`x-target-provider(s)`, `x-gateway-target-provider`) in base al protocollo della richiesta e riscrive il model selector nel body (`rewriteBodyModelForProtocol`). Cache dei routing per model (`attemptRoutingCache`).
2. **Build tentativi**: `createRouteExecutionPlan` → `attempts[]`.
3. **Per ogni tentativo**:
   - `prepareUpstreamCredentialAttempt`: risolve il **target provider+credential** (da header, da model selector, o da piano), applica body/usage adaptation (`usageAwareOpenAiChatAttemptBody`: forza `stream_options.include_usage`; strip di `thinking`/`reasoning_split` non supportati), inietta header oauth locali (`withClaudeCodeOauthBetaHeader` per il plugin OAuth), seleziona le credential con **weighted priority + spillover** (`selectProviderCredentials`: esclude quelle in cooldown/limite, ordina per priority→utilization→weight; se tutte le top sono ≥80% utilization, riordina per utilization = spillover), setta `x-target-providers` (lista candidati) e `x-ccr-provider-credential-chain`.
   - **Fetch**: `fetchWithSystemProxy` (rispetta il system proxy e il NO_PROXY).
   - **Fallback**: se `shouldFallbackAfterStatus(status, mode)` e c'è un prossimo tentativo → delay (`retryDelayAfterStatus`, rispetta `Retry-After`; `retryDelayAfterNetworkError` con backoff), `drainResponseBody`, e passa al tentativo successivo.
   - **Esito**: trace attempt outcome, `recordProviderCredentialOutcome`.
4. Gli header di risposta includono metadati: `x-ccr-fallback-attempts`, `x-ccr-fallback-failures`, `x-ccr-fallback-model`, `x-ccr-provider-credential-chain`.

`UpstreamRequestError` è l'errore tipizzato che trasporta `attempt` + `failedAttempts` — la pipeline lo usa per loggare con il contesto.

### 5.4 `core-runtime/supervisor.ts` (840 righe) — supervisione del subprocess
Gestisce il **core gateway come child process**:
- `spawnGatewayProcess(config, gatewayConfig, upstreamProxyUrl, runtimeId, coreAuthToken)`: risolve il runtime Node (nativi, con `ELECTRON_RUN_AS_NODE` se l'app gira in Electron, probing di `better-sqlite3` per compatibilità ABI!), scrive un **preload file** (`gateway-proxy-preload.cjs`) che **patcha `globalThis.fetch`** con ProxyAgent undici + timeout + NO_PROXY-aware; spawna `node --require <preload> <gateway-bootstrap>` con `serialization:"advanced"` e canale IPC.
- `monitorGatewayConfigAcceptance`: promessa che si risolve quando il child risponde `gateway:config-accepted` (timeout 5s, gestione exit/error con output catturato, `gatewayChildOutput` WeakMap con tail 4000 byte).
- `writeManagedCoreGatewayMarker` / `readManagedCoreGatewayMarker` / `stopPreviousManagedCoreGateway`: **marker di runtime** (file + stato persistito) con `pid`/`runtimeId`/`startedAt`; prima di farne partire uno nuovo, **termina il precedente** se è lo stesso runtime (health check su `/health` + kill SIGTERM→SIGKILL).
- `generateCoreGatewayAuthToken`: token da 32 byte base64url per l'auth interna.
- `gatewayNetworkEndpoints`: enumera gli indirizzi LAN (IPv4 privati, filtra interfacce virtuali: docker, veth, tailscale, wsl, vpn, loopback...) per esporre l'endpoint del gateway.
- `shouldRunGatewayRuntime` / `shouldServeGatewayRequest` / `shouldRunUnifiedServer`: predicate di decisione su cosa avviare.

### 5.5 `core-runtime/gateway-bootstrap.ts` (115 righe) — entry del child
Il child process attende un messaggio IPC `gateway:start` (config + entry module). Fa una cosa geniale: **installa un config file virtuale** — patch di `fs.existsSync/readFileSync/writeFileSync/renameSync` così che il core gateway legga la config da memoria (path virtuale `.ccr-gateway-config-<pid>`), e **blocca** scritture/rename su quel path (fail-fast se il gateway tenta di modificare la config gestita). Poi `require(gatewayEntry)` e conferma `gateway:config-accepted`. Sul `disconnect` del parent → exit.

**Forza**: configurazione iniettata senza toccare disco (nessun file config temporaneo), immutabile per il child. Pattern "config injection via fs-mock" replicabile.

### 5.6 `core-runtime/config-compiler.ts` (812 righe) — traduzione config
`compileCoreGatewayConfig(config, rawTraceSyncToken, billingSyncToken, coreAuthToken, browserWebSearchMcpIntegration, upstreamProxyUrl)` → `Record<string, unknown>` = la config del core gateway:
- Espande i provider (via `toCoreGatewayProviders`), aggiunge provider generati dai tool fusion.
- **Inietta i plugin OAuth locali** (Claude Code/Codex/Grok/Kimi login → provider) con runtime defaults: legge il token OAuth dalla macchina (`readClaudeCodeOauth`, `readCodexAuth`, `resolveGrokAuth`, `resolveKimiAuth`), e genera provider plugin `{auth: {headers: {authorization: "Bearer ..."}}, providerName, key}`.
- Compila i **virtual model profiles** con i tool fusion (web search, vision, media), rimuovendo i limiti del tool loop (`maxToolCalls/maxTurns = MAX_SAFE_INTEGER`).
- Assembla MCP servers: built-in tool artifacts + media + toolhub + external (agent + plugin).
- Config auth: `static_api_key` con header `x-ccr-core-auth` e token da env `CCR_CORE_GATEWAY_AUTH_TOKEN`.
- `rawTrace`: policy per il sync delle trace verso il gateway CCR.

**Insieme a `supervisor.ts` e `gateway-bootstrap.ts` forma il pattern "wrapping a third-party gateway"**: config dichiarativa esterna → config del core → IPC → subprocess isolato con fetch patchato. Elegante.

### 5.7 `context-archive/` — il "contesto compattato" persistente
**`protocol.ts` (798 righe)** — puro, manipolazione protocollo:
- Rilevamento segnale di compattazione: `hasExplicitCompactSignal` (header, `context_management.edits` type `compact_*`, prompt auto-compact di Claude Code riconosciuto da stringhe fisse, metadata `ccr_context_compact`).
- `appendTask` (per protocollo): append di un messaggio user al body (`anthropic`/`chat` → `messages[]`; `responses` → `input[]`).
- `compactHandoffTask` / `archiveHandoffFooter`: il **task di handoff** che viene mandato al modello compattato e il **footer** "CCR ARCHIVED HISTORY ACCESS" (con archive_id, session_token, generazione) da appendere alla risposta.
- `historyReplayTask`: il task per il replay di una domanda storica.
- `appendArchiveFooterToResponse` / `appendFooterToSse` / `renderCodexCompactArchiveResponse`: **iniezione del footer nel flusso di risposta**, sia JSON che SSE (split per eventi, trova l'evento terminale, inietta i blocchi prima di `message_stop`/`response.completed`/`[DONE]`), sia per compattazione Codex (`compaction` item).
- `extractArchiveAssistantText` / `collectProtocolText` / `collectSseProtocolText`: estrazione del testo della risposta da ogni protocollo (per estrarre l'answer del replay).
- `assertAppendableTurn`: non si può appendere un task se la richiesta termina con un tool call non risolto.

**`store.ts` (306 righe)** — storage SQLite:
- Tabella `archive_snapshots`: `archive_id PK, session_id, generation, parent_archive_id, request_id UNIQUE, protocol, method, path, body BLOB, body_sha256, replay_headers_json, route_json, token_hash, status(pending|ready|failed), created_at, expires_at`, indice `(session_id, generation DESC)`, `UNIQUE(session_id, generation)`.
- `create`: transazione → calcola `generation = max+1` e `parentArchiveId` (lineage), inserisce, poi `extendLineageExpiry` + `prune` (ritenzione: scadenza, max snapshots, max bytes, **proteggendo il lineage del nuovo snapshot**).
- `finalize` (scrive la route usata: credential chain, logical provider, protocollo, model), `fail`, `get`, `lineage` (remonta la catena parent).
- Hardening file: chmod 700 dir / 600 db+wal+shm, WAL mode.

**`context-archive.ts` (840 righe)** — il servizio:
- `ContextArchiveService` con `stores` per DB file. `createSnapshot` genera `archiveId` (`arc_<random>`), `sessionToken` (32 byte, salvato **hashato** con SHA-256 — il plaintext esiste solo nel footer), verifica dimensione max.
- `ask()`: **l'API di interrogazione del passato**: verifica esistenza, scadenza, status ready, **token con `timingSafeEqual`**; poi scende il **lineage** (max 32 generazioni): per ogni snapshot fa un **replay** (append task → esecuzione upstream) e se la risposta è "insufficiente" (frase pattern-matching) prova il parent; altrimenti restituisce la prima risposta sufficiente.
- `replayArchiveSnapshot`: esegue via `executor` (injectato dalla pipeline) con AbortController+timeout.
- Espone un **server MCP** (`handleContextArchiveMcpRequest`, path `/__ccr/context-archive/mcp`, JSON-RPC 2.0): `initialize`, `tools/list` (tool `ccr_history_ask`), `tools/call`. Tool schema richiede `archive_id`, `session_token`, `task`.
- Attivazione **per-API-key e per-profilo**: `contextArchiveConfigForApiKey` attiva l'archive se il profilo ha `managedCompact` (managed compaction → obbligatorio), oppure se globalmente abilitato.
- `prepareContextArchiveRequest`: hook della pipeline — se c'è segnale di compact → snapshot + body riscritto (task handoff) + path rewrite.

**Questo è il pattern "compaction-aware gateway" più avanzato del sistema**: quando il client compatta, CCR non lascia perdere il contesto: archivia, fa produrre un handoff, append il footer (che contiene le credenziali per riinterrogare il passato via tool MCP), e il tool permette di **ri-chiedere al modello originale** (replay con route originale) qualsiasi dettaglio pre-compattazione. È memoria a lungo termine dentro un proxy LLM. Da qui può nascere la "memoria" di Nexus.

### 5.8 `features/` — compat layer per client specifici
- **`anthropic-response-model.ts`**: riscrive il campo `model` nel `message_start` della risposta SSE Anthropic (per far credere al client di aver parlato col suo modello). `shouldRewriteAnthropicMessageStartModel`.
- **`codex-patch-bridge.ts`**: per Codex, inietta nel system prompt l'istruzione di usare `virtual_apply_patch` (grammar Lark della sintassi patch inclusa in `internal/shared.ts`), inietta il tool virtuale `virtual_apply_patch` nel body, e intercetta la risposta per collegare il tool a un'applicatore di patch reale. Strumento `virtualApplyPatchLarkGrammar` con grammatica completa `*** Begin Patch/*** Update File/*** End Patch`.
- **`codex-multi-agent-bridge.ts`**: bridge per il multi-agent di Codex.
- **`cursor-compat.ts`**: adatta body OpenAI-compat per Cursor (fallback system prompt etc.).
- **`context-archive-continuation.ts`**: quando il tool `ccr_history_ask` viene invocato dal client nel mezzo di una conversazione, gestisce il ciclo: esegue il replay (via `replayContextArchive`), risolve il risultato e lo converte in un secondo messaggio verso il client (chiamata tool → risultato).
- **`hosted-web-search/`**: modulo completo di web search ospitato: `discovery` (rileva se il client chiede web search), `request-transform` (converte la richiesta di web search in una search reale), `response-transform` (converte i risultati in formato atteso), `evidence` (mapping risultati), `sse` (iniezione nel flusso SSE), `index` (orchestrazione). Con `WebSearchProtocolContext`, record browser `BrowserWebSearchProtocolRecord`.
- **`model-discovery.ts` (696 righe)**: la "menu dei modelli" esposta al client:
  - `GET /models` e `GET /v1/models` risposti localmente (anche endpoint OpenAI-compat `object:"list"`).
  - Per Claude Code/Claude App genera una risposta **stile Claude** (`createClaudeAppGatewayModelsResponse`) con `capabilities` (context window, `1m` variant `[1m]`, web search, input tokens), calcolando `max_input_tokens` da catalogo + metadati provider.
  - `prepareClaudeCodeDiscoveredModelRequest` / `prepareClaudeAppDiscoveredModelRequest`: **rewrite del model richiesto → model provider** nel body di `/v1/messages` (mappa gli id pubblici client ai selettori interni `provider/model`).

### 5.9 `auth/api-key-authorizer.ts` + `limits/window-limiter.ts`
- `authorize(request, response, config)`: carica API keys (cache 1s), confronta token con **`timingSafeEqual`**, verifica scadenza. Ritorna `{ok, apiKey}`.
- `reserveApiKeyLimits`: stima uso (tokens da `messages/system/tools` ≈ chars/4, immagini contate ricorsivamente), costruisce le regole (`limitRules`: rpm/rph/rpd, tpm/tph/tpd, ipm/iph/ipd, quota), controlla i **window counters in-memory** (`Map`, pruning dei scaduti), e se oltre → 429 con dettagli strutturati; altrimenti incrementa e passa.
- I counter sono **in-memory con finestra scorrevole per bucket** (`windowStart = floor(now/windowMs)*windowMs`), ritenzione 2 finestre.

### 5.10 `http/`
- `io.ts`: utilità — `inferGatewayClient` (da user-agent o header), `readAuthToken`, `forwardHeaders` (con deny-list `connection`/`host`/`upgrade`/`x-ccr-core-auth`), `stripLocalGatewayAuthHeaders`, `filteredResponseHeaders` (deny `content-encoding`/`transfer-encoding`), **`formatUpstreamErrorForLog`** (catena errori causa-effetto, redazione credential con regex, fase infierita: dns/tls/connect/headers/body/aborted), `closeServer` (con timeout e `closeAllConnections`).
- `body.ts`: `parseJsonObjectSafe`, `takeJsonObject`, `serializeJsonBody`, `serializeJsonBodyWithModel`, `releaseJsonObject`.
- `request-handler.ts`: routing HTTP del server: OPTIONS/CORS, path speciali (`/__ccr/billing-usage-sync`, `/__ccr/raw-trace-sync`, remote-control, MCP paths: browser automation, context archive, media tools, network capture), plugin routes, `/health`, `/`, `/v1/messages/count_tokens` (calcolo token locale), altrimenti proxy. **Ogni endpoint speciale è authz-indipendente o authz-protetto in base al rischio.**

### 5.11 `claude-code-router-plugin.ts` (parte restante, righe ~1404+)
È l'oggetto "router" esposto alla pipeline (`routeRequest`). Fasi:
1. **Enricher agent**: per il client Claude Code inietta istruzioni nei tool (`agent`/`task`/`workflow`: istruzioni per subagent routing con tag `<CCR-SUBAGENT-MODEL>Provider/model</CCR-SUBAGENT-MODEL>`), istruzioni ToolHub; rimuove l'header di billing `x-anthropic-billing-header` dal system (e rileva subagent da `cc_is_subagent`).
2. `sessionId`, `tokenCount` (conteggio chars→tokens).
3. **Custom router** opzionale (`CUSTOM_ROUTER_PATH`): modulo JS locale caricato a runtime (deve stare dentro CONFIGDIR, estensione .js/.cjs/.mjs, no package specifier) che ritorna un model selector.
4. `resolveConfiguredRouteDecision`: **policy chain** in ordine di priorità:
   `custom → builtin-agent-claude-code-subagent → profilePolicies → rules → builtin-agent-claude-code-subagent-env → client-model → builtin-agent → default`.
   Le policy sono `RoutePolicy[]` valutate dal `RoutePolicyEngine` (primo match vince). `clientModelDecision` è l'esplicito del client che può override della route builtin, con eccezioni (subagent). Le decisioni si **fondono** (`mergeConfiguredRouteDecisions`) per regole che aggiungono rewrites.
5. Regole: `condition` (operatori `==`,`!=`,`>`,`>=`,`<`,`<=`,`contains`,`contains-deep`,`not-contains`,`starts-with` su path `request.header.*`, `request.body.*`, `request.auth.*`), `model-prefix` (model richiesto inizia con pattern), `script` (via worker).
6. Rewrite applicati dopo la decisione, con trace.

---

## 6. `agents/` — Adattatori per i client agent

### 6.1 `local-providers/` — importa il login locale di ogni CLI come provider
Il pattern: **scansiona il login locale della CLI → genera un candidato provider (`LocalAgentProviderCandidate`) → lo importa come `GatewayProviderConfig` + `providerPlugins` (auth bearer) → lo fa usare dal gateway**.

- **`shared.ts`**: helper comuni — `missingCandidate`, `providerPayload` (build del payload deep-link con API key sentinella `ccr-local-agent-login`), **`bearerAuthPlugin`/`apiKeyAuthPlugin`** (generatori di provider plugin `{auth:{headers:{authorization:"Bearer ..."}, removeHeaders:["x-api-key"], strict:true}, key, providerName}`), `readOauthTokenSetFields` (solo campi root, NON ricorsivo — per evitare di importare i token MCP `mcpOAuth`), parser JSONC.
- **`claude-code.ts` (437 righe)**: la gemma. `scanClaudeCodeLogin` trova l'OAuth di Claude Code su macOS:
  - Keychain: calcola il **nome del servizio** esatto (formato `Claude Code<oauthSuffix>-credentials<configSuffix>` dove `configSuffix = -sha256(NFC(configDir)).slice(0,8)`, verificato contro 2.1.220), account = `$USER`, tre tier di lettura (atteso → discovery via `security dump-keychain` → lookup senza account per pre-2.1), leggendo solo `claudeAiOauth` root-level (per non importare `mcpOAuth`).
  - Fallback file: `~/.claude/.credentials.json` e varianti (rispetta `CLAUDE_CONFIG_DIR`/`CLAUDE_SECURESTORAGE_CONFIG_DIR`).
  - `importClaudeCodeProvider`: provider `Claude Code API` → `https://api.anthropic.com`, protocol `anthropic_messages`, account connector che legge `/api/oauth/usage` (mapping quota 5h/7d, Opus/Sonnet, extra usage credits — **i meter sono espressioni string**).
- **`codex.ts` (932 righe)**, **`grok.ts` (910)**, **`kimi.ts` (780)**, **`opencode.ts` (551)**, **`zcode.ts` (327)**: stessa architettura per i rispettivi login locali (Codex auth, Grok auth, Kimi, OpenCode, Zcode), con probe dei modelli, metadati model (context window, pricing), e plugin OAuth con refresh.
- **`service.ts`**: orchestratore — `getLocalAgentProviderCandidates()` (filtra `missing`), `importLocalAgentProvider`, `probeLocalAgentProvider`. Espone anche i getter `read*Auth` per il config-compiler del core.

### 6.2 Altri agent
- `claude-app/`: gateway per l'app desktop Claude (`gateway-service.ts`, `gateway-routes.ts` = mapping modelli Claude app → provider, `cdp.ts` = Chrome DevTools Protocol, `launch.ts`, `vm-storage.ts`).
- `claude-code/environment.ts`: env di lancio di Claude Code (abilita model discovery: `CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1`).
- `codex/`, `opencode/`, `kilo/`, `pi/`, `zcode/`: profile-config e app-launch per ogni CLI.
- `bot-gateway/`: gateway bot (WeChat/Feishu/DingTalk/Slack/Telegram...) con `qr-login-service`, `handoff-scan-service` (handoff telefono: bluetooth/wifi/idle), `env`, `sdk-import`.
- `cdp-client.ts`: client CDP generico.
- `request-enricher.ts` (20 righe): interfaccia `enrich(request)` per mutare la richiesta per agent specifici (usata dal plugin router).

**Forza**: l'idea di **trattare il login locale di una CLI come un provider** è brillante: invece di chiedere API key, CCR riusa il login già fatto sul computer. Per Nexus: scanner dei credential store + generazione plugin di auth.

---

## 7. `mcp/`, `media/`, `plugins/`, `proxy/`, `usage/`, `observability/`, `storage/`, `profiles/`, `platform/`, `runtime/`, `web/`

### 7.1 `mcp/`
- `fusion-config.ts`: configurazione dei **virtual model "Fusion"** (aliases, tool web search/vision, MCP server fusion, artifact tool). `fusionBuiltinToolArtifacts` costruisce i provider/tool built-in per i virtual model.
- `toolhub-mcp.ts` + `toolhub-config.ts`: **ToolHub** = un MCP server che *risolve e invoca* tool da altri server MCP (un meta-risolutore di tool): `tools/search`, `tools/resolve`, `tools/invoke`, con LLM config per la risoluzione. Inietta istruzioni nel system prompt via plugin.
- `browser-web-search-proxy-mcp.ts`, `network-capture-mcp.ts`, `media-tools-proxy-mcp.ts`, `grok-media-mcp.ts`, `fusion-tool-fallback-mcp.ts`, `fusion-vision-mcp.ts`: proxy MCP verso funzionalità (browser search, network capture, media generation, vision, fallback tool).
- `tool-discovery.ts`: discovery dei tool esposti.

### 7.2 `media/`
Servizio completo generazione media: `contracts.ts`/`models.ts` (tipi job/artefatti), `storage.ts` (persistenza artefatti con TTL), `service.ts` (orchestrazione job, concurrency image/video), `executors.ts` (chiamate API: `openai_image_generations`, `openai_video_generations`, `xai_video_generations`), `tools.ts` (esposizione come tool MCP: `image_generate*`, `video_generate*`, `media_job_get/cancel`).

### 7.3 `plugins/`
- `service.ts`: `GatewayPluginService` — carica plugin con **permessi e superfici** dichiarati, registra gateway routes / proxy routes / http backends / account connectors / virtual model profiles / sqlite store, e il `coreGatewayConfig` (config che plugin possono contribuire al core gateway). `matchGatewayRoute`/`handleGatewayRoute`.
- `backend-service.ts`: gestisce processi backend dei plugin (spawn/stop).
- `marketplace.ts`: marketplace plugin.

### 7.4 `proxy/`
- `service.ts` (1889 righe): **proxy HTTP/HTTPS** (trasparente o gateway): server HTTP + tunnelling CONNECT HTTPS, forward con `http.request`/`https` agent, upstream proxy (custom), **network capture** (registra exchange con body sampler/truncation, `ProxyNetworkExchange`), routing verso il gateway (`shouldRouteToGateway` per i target), gestione certificato CA (installazione macOS/Windows, fingerprint), system proxy (macOS `networksetup`).
- `certificates.ts`, `system-proxy.ts`, `system-proxy-fetch.ts` (fetch con system proxy + NO_PROXY), `undici-proxy-agent.ts` (module resolvable per il preload).

### 7.5 `observability/`
- `request-log-*.ts`: store dei log richiesta (SQLite, admission control, sampling success, body capture policy `all|errors|none`, limiti body, worker asincrono), `request-log-model.ts` (estrazione model richiesto/risposto).
- `route-trace.ts`: **RequestRouteTraceRecorder** — vedi §8.
- `raw-trace-sync.ts`: sincronizzazione delle trace raw dal core gateway (endpoint `/__ccr/raw-trace-sync`), con `createBodySampler`, `requestLogSampled`, `shouldRecordRequestLogs`.
- `sensitive-headers.ts`: lista header sensibili.

### 7.6 `usage/`
- `store.ts`: `recordGatewayUsageCapture` (usage capture in coda), `normalization.ts` (normalizzazione modelli/token), `model-attribution.ts` (attribuzione di un model a provider/modello logico), `billing-sync.ts` (`GatewayBillingSynchronizer`, endpoint `/__ccr/billing-usage-sync`).

### 7.7 `storage/sqlite-native.ts`
Astrazione database: `createBetterSqliteDatabase` risolve il binding nativo `better-sqlite3` (con fallback e probing compatibilità) e lo espone con API tipizzata (`pragma`, `prepare`, `transaction`).

### 7.8 `profiles/`
- `service.ts`, `launch-service.ts`, `launch-core.ts`: gestione **profili agent** (Claude Code/Codex/Grok/...) — un profilo = set di modelli + env + configurazione del client; `applyProfile` scrive il config del client CLI (`settings.json` di Claude Code, `config.toml` di Codex); `open` lancia il client.
- `api-key.ts`: mappa profilo → API key interna (`profile:<id>`).

### 7.9 `platform/`, `runtime/`, `web/`, `entrypoints/`
- `platform/socket-compat.ts` (patch `typeof`, `SO_TYPE`), `windows-*.ts` (app discovery, system).
- `runtime/app-paths.ts` (paths), `runtime/desktop-app.ts` (rilevamento app desktop).
- `web/management-server.ts`: server web di gestione (UI + RPC auth `CCR_WEB_AUTH_TOKEN`), `entrypoints/server.ts`: CLI `ccr-core-server` (`--host/--port/--no-gateway/--gateway`).

---

## 8. Osservabilità: `RequestRouteTrace` — il sistema di traccia

**Percorso**: `observability/route-trace.ts` + tipi in `contracts/app.ts` (righe 1999-2085).

La traccia è una lista di **hop** (`RequestRouteTraceHop`), ciascuno con:
- `phase`: `ingress|compatibility|routing|capability|enrichment|planning|attempt|core|outcome`
- `kind`: `snapshot|decision|mutation|attempt|outcome`
- `changes`: diff strutturati `{scope: body|headers|routing|url, path (JSON-pointer), operation: add|remove|replace, before?, after?, redacted?, truncated?}`
- `decision`: `{reason, source, ruleId, ruleName, policyId, diagnostics[]}`
- `outcome`: `{statusCode?, error?, fallbackReason?, retryDelayMs?}`
- `target`: `{model?, provider?, protocol?, credentialId?, credentialCandidates?}`

**Design key**: il recorder **non guarda mai il body** — i siti di mutazione *riportano* i propri cambi. Costo ∝ cambi riportati, non dimensione richiesta. Budget rigidi: max 64 hop, 64 changes/hop, 256KB trace, preview con troncamento per stringhe/array/oggetti, **redazione** di campi sensibili (authorization, api-key, token, cookie) per nome e per path. `finish({captureBodyValues})` può sopprimere i valori body dal trace.

Ogni speranza di "capire cosa è successo a una richiesta" di Nexus dovrebbe copiare questo modello: **trace come lista di hop con diff tipizzato, prodotta dai punti di mutazione, con budget e redazione**.

---

## 9. I 10 pattern/architetture più importanti da cui Nexus può imparare

1. **Formato standard interno a 3 livelli (client-protocollo / provider-capability / core-provider)**: il sistema non lavora mai con "provider generici", ma con `GatewayProviderCapability` (protocollo + baseUrl) e una **fallback chain di protocolli** (`providerProtocolForClientProtocol`). Nexus deve definire il proprio set di protocolli e la catena di preferenza, e ragionare a livello protocollo, mai provider.

2. **Gateway "spesso" (routing/rewrite/trace) davanti a un core gateway "magro" in subprocess isolato**: pattern `supervisor.ts` + `gateway-bootstrap.ts` (config iniettata via fs-virtuale, immutabile, auth token interno, fetch patchato per proxy/timeout, marker di runtime con riconoscimento dell'istanza). Non serve riscrivere un provider gateway da zero: si incapsula e si arricchisce.

3. **Context archive (memoria post-compattazione)**: snapshot immutabili SQLite con lineage (parent), generazione, token hashato, ritenzione multi-vincolo, handoff-task che produce un footer con credenziali di accesso, e un tool MCP (`ccr_history_ask`) che fa **replay del modello originale** sulla snapshot per rispondere a domande sul passato. È letteralmente una memoria a lungo termine inserita in un proxy — Nexus può adottare lo schema (snapshot + lineage + replay su route originale).

4. **Route trace a hop con diff tipizzato prodotto dai punti di mutazione**: osservabilità a costo proporzionale ai cambi (mai a dimensione body), con budget, redazione e soppressione dei valori body per policy. Il prerequisito per "spiegare qualsiasi decisione di routing".

5. **Policy engine + config compiler con diagnostica strutturata**: le regole (condition/model-prefix/script) vengono **compilate a (ri)avvio** in strutture pronte con `RouteDiagnostic[]` (codici tipizzati), il runtime le valuta come **chain di policy con primo match**, e le decisioni si fondono per sovrapposizione (rewrites cumulativi). Fallback ridotti ai modelli validi a compile-time.

6. **Rewrite JSON-path sicuro con diff**: DSL `request.body.<path>` / `request.header.<name>` con operazioni array, whitelist di header protetti, blocco prototype-pollution, valori letterali parsati, e ogni rewrite che produce il proprio `RequestRouteTraceChange`. La mutazione è sempre "compilata una volta, applicata con tracciabilità".

7. **Naming interno parsabile negli header**: `provider::protocol`, `provider::protocol::cred:<slug>`, `x-target-provider(s)`, `x-ccr-provider-credential-chain`, `x-ccr-route-reason`, `x-ccr-routed-model`, `x-ccr-provider-protocol`. Il *protocollo, il provider, la credential e la motivazione* viaggiano nella richiesta in forma parsabile — il che rende l'intero sistema introspettivo e testabile (headers come traccia).

8. **Credential pool con cooldown, spillover e weight**: più chiavi per provider; selezione con priorità→utilizzazione→peso, **spillover** quando le top sono ≥80% di utilizzo, cooldown su 401/403/429/5xx, contatori di finestra in-memory. Il rate limiting per API key e per credential riusa la stessa infrastruttura di window counter.

9. **Login locale della CLI = provider**: scanner dei credential store (Keychain macOS con nome servizio calcolato deterministicamente, file `.credentials.json` rispettando le env dir) che genera `providerPlugins` di auth (`Bearer`/`x-api-key`, removeHeaders, strict) senza richiedere chiavi manuali. Il gateway riusa il login già fatto dall'utente.

10. **Compat layer per client in `features/` come anelli di stream**: ogni client esotico (Cursor, Codex multi-agent, Codex patch-bridge, Claude App, context archive continuation, web search hosted) è un **modulo `prepare*` + `*ResponseStream`**: transform sul request body e Transform stream sul response. La pipeline è un insieme ordinato di "anelli" opzionali, con header diagnostici (`x-ccr-*`) che dichiarano quale anello ha agito. Aggiungere un client = aggiungere un anello, non toccare il core.

---

## Nota metodologica

Analisi statica completa dei file prioritari richiesti (contratti, config, runtime-topology, protocol-endpoints, rewrite, pipeline, executor, shared, supervisor, bootstrap, gateway-service, presets, context-archive e store, plugin router, model-registry, config-compiler routing, api-key-authorizer, window-limiter, request-handler, io, credential-pool, model-discovery, core config-compiler, route-trace) e campionatura approfondita del resto (local-providers, mcp, media, proxy, observability, usage, storage, plugins, profiles). Nessun file è stato modificato.

Limiti dell'analisi: `config.ts` e `claude-code-router-plugin.ts` letti per intero nelle parti strutturali ma non riga-per-riga in coda (funzioni di utility ripetitive); il contenuto esatto di `@the-next-ai/ai-gateway` (dipendenza esterna non presente) è inferito dai punti di contatto (config compile, endpoint `/health`, `gateway-runtime.json`).
