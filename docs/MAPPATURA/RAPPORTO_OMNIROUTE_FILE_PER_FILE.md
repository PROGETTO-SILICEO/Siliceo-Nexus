# RAPPORTO OMNIROUTE — ANALISI FILE PER FILE

**Oggetto:** `omniroute-ref` — "Unified AI router with 291 providers, RTK+Caveman compression, auto fallback"
**Tipo analisi:** statica, read-only (nessun file modificato)
**Percorsi analizzati:** `src/domain/` (24 file), `src/server/`, `src/lib/quota/` + meccanismi core, `open-sse/services/compression/` (RTK+Caveman)
**Data:** 2026-08-10
**Autore:** Sempre (analisi per il progetto Nexus — LLM gateway)

---

## 1. VISIONE D'INSIEME

OmniRoute è un proxy/gateway LLM che espone API compatibili OpenAI (Chat Completions, Responses, SSE) verso **291 provider** (gratuiti OAuth, no-auth, API key) con un'architettura a strati:

```
┌─────────────────────────────────────────────────────────────┐
│ src/server/authz (middleware: classify → policy → stamp)    │
├─────────────────────────────────────────────────────────────┤
│ src/domain (logica pura di business: routing, quota, cost)  │
├─────────────────────────────────────────────────────────────┤
│ src/lib (meccanismi: cache, quota engine, idempotenza, ...) │
├─────────────────────────────────────────────────────────────┤
│ open-sse (executor, compression RTK+Caveman, usage fetcher) │
└─────────────────────────────────────────────────────────────┘
```

Il layer `src/domain` è il cuore logico **indipendente da framework** (lazy-loading di dipendenze via import dinamico, JSDoc per i tipi senza fase di build TS obbligatoria). I principi trasversali dominanti:

- **Fail-open di default** per tutto ciò che tocca infrastruttura esterna (un errore di quota/cache/DB non deve mai bloccare una richiesta legittima).
- **In-memory cache + SQLite persistente** come coppia indissolubile (il DB è backup e surrogato dopo restart; la memoria è il hot path).
- **Meccanismi auto-riparanti**: quota cache con TTL e lazy reset, lockout con finestra temporale, assessment proattivo dei provider, self-healing dei combo.
- **Telemetria pervasiva**: header `X-OmniRoute-*` su ogni risposta (costo, latenza, token, fallback, decisione di routing).

---

## 2. ANALISI `src/domain/`

### 2.1 `fallbackPolicy.ts` — Catene di fallback dichiarative

**Cosa fa.** Registra, per ogni modello, una lista ordinata di provider di fallback (`FallbackEntry[]`: provider + priority + enabled). Espone `registerFallback`, `resolveFallbackChain(model, excludeProviders)`, `getNextFallback`, `hasFallback`, `removeFallback`, `getAllFallbackChains`, `resetAllFallbacks`. La cache in-memory (`Map`) viene idratata da SQLite con lazy-loading (`ensureLoaded`); ogni write è best-effort (se il DB non è pronto, la memoria continua a funzionare). `excludeProviders` permette di escludere provider già tentati — fondamentale per non ripetere un fallback fallito.

**Pattern architetturali.**
- *Cache-aside con persistenza* (read: DB→memoria; write: memoria→DB best-effort).
- *Configurazione dichiarativa* (una catena è dato, non codice).
- *Lazy hydration* per tollerare la fase di build senza DB.

**Punti di forza / lezione per Nexus.** Il fallback deve essere **dato**, non logica sparsa: una tabella "modello → lista ordinata di provider" risolta con un singolo lookup. L'idea di `excludeProviders` (i provider già tentati in questo request-cycle) è il seme dell'anti-loop. Aggiungerei: TTL di *stato* per provider falliti (il fallback "sano" deve sapere quando un provider è tornato disponibile — Nexus dovrebbe abbinare fallbackPolicy a una memoria di *health* con scadenza).

### 2.2 `policyEngine.ts` — Verdetto centralizzato prima dell'inoltro

**Cosa fa.** Due implementazioni nello stesso file:
1. `evaluateRequest(request): PolicyVerdict` — pipeline ordinata di fasi: **lockout** (per IP) → **budget** (per API key) → **fallback chain resolution**. Restituisce `{ allowed, reason, adjustments, policyPhase }`. `evaluateFirstAllowed(models)` prova i modelli in ordine e restituisce il primo permesso (utile quando il chiamante offre alternative).
2. `PolicyEngine` (classe) — policy dichiarative (`{id, type, priority, conditions.model_pattern, actions}`) con match glob su pattern modello, tipi `routing` (prefer provider), `access` (block model), `budget` (max_tokens), ordinate per priorità.

**Pattern architetturali.**
- *Chain of responsibility* (policy ordinate per priorità, short-circuit su block).
- *Pattern matching glob* (`*` → regex) per applicare policy a famiglie di modelli.
- *Verdict immutabile* con fase di rifiuto per debugging.

**Punti di forza / lezione per Nexus.** Il concetto chiave: **tutte le decisioni convergono in UN verdetto prima di toccare la rete**. Ogni blocco riporta la *fase* (lockout/budget/policy) → osservabilità istantanea del perché una richiesta è stata rifiutata. Nexus dovrebbe avere un `PolicyVerdict` unico con `policyPhase` e `reason` obbligatori.

### 2.3 `tagRouter.ts` — Routing per tag di richiesta

**Cosa fa.** Normalizza i tag di routing da `metadata.tags` (array o CSV), supporta due modalità di match (`any`/`all`) e confronta i tag richiesti con i tag della connessione provider. Pure functions, nessun side-effect. Edge case gestiti: tag vuoti → true (match permissivo); connessione senza tag → false (nessun match); case-insensitive, dedup.

**Pattern architetturali.** *Strategy/decoration del routing*; *normalizzazione input robusta* (fronte di validazione che accetta sia array che stringhe CSV, come il resto del codebase).

**Punti di forza / lezione per Nexus.** I tag sono un modo elegante per dire "questo modello va instradato verso connessioni che supportano X" senza regole astruse. Il default permissivo (richiesta senza tag → match) è la scelta giusta per non rompere i client esistenti. Nexus: mantenere il tag-match come **filtro pre-routing** a costo zero.

### 2.4 `comboResolver.ts` — Selezione modello da "combo"

**Cosa fa.** Dato un *combo* (gruppo di modelli con strategia), risolve quale modello usare. Strategie: `priority` (primo modello), `round-robin` (contatore persistente per combo), `random` (weighted), `least-used` (dal contesto `modelUsageCounts`). `getComboFallbacks` restituisce gli altri modelli del combo come catena di fallback naturale.

**Pattern architetturali.** *Factory/strategy pattern* (switch su `strategy`); *selezione pesata* (random con cumulativa); *stato minimo deterministico* (contatori round-robin in memoria).

**Punti di forza / lezione per Nexus.** Il combo è la primitiva di routing più flessibile: **un fallback con strategie multiple** (rotation, load balancing, weighted). `getComboFallbacks` rende il fallback *strutturale* (i fallback sono gli altri membri del combo) invece che una lista separata da mantenere. Criticità da evitare: i contatori round-robin in memoria non sono condivisi tra processi (stessa lezione del `globalThis` in quotaCache) — Nexus deve decidere subito se servono su più istanze.

### 2.5 `costRules.ts` — Budget e spesa per API key (631 righe)

**Cosa fa.** Gestione completa dei budget per API key: configurazione di limiti daily/weekly/monthly in USD, *finestre di reset* calcolate in UTC (`getBudgetWindow`: daily/weekly/monthly con ora di reset configurabile), normalizzazione robusta della config (intervalli validi, regex `HH:MM`), warning di soglia (`warningThreshold`, once-per-period), log di reset (`saveBudgetResetLog`), e `checkBudget(apiKeyId, additionalCost)` che fa la **proiezione** (periodUsed + costo stimato) prima di autorizzare. La spesa è registrata via `spendBatchWriter` (batch asincrono) e letta sia da SQLite sia dai pending del batch — questo rende i conti coerenti in tempo reale. `syncAllBudgetSchedules` allinea tutti i budget al clock e persiste i reset scaduti.

**Pattern architetturali.**
- *Time-window arithmetic* pura (funzioni deterministiche con `now` iniettato → testabile).
- *Projected budget check*: `periodUsed + additionalCost > limit` (autorizzazione a costo stimato, non a posteriori).
- *Batch writer + read-your-writes* (spesa non ancora flushata è già conteggiata nei totali).
- *Normalizzazione difensiva* (nessun valore proveniente dal DB viene fidato: tutto passa da `toNumber`/`normalize*`).

**Punti di forza / lezione per Nexus.** Tre lezioni forti:
1. **Il budget si controlla in proiezione**, non a consuntivo: `additionalCost` è il parametro che Nexus deve esporre a chi chiama l'engine.
2. Le finestre di reset in UTC con ora configurabile e il **rollover lazy** (il reset viene "scoperto" quando si legge) eliminano ogni cron job per il reset dei budget.
3. Il warning di soglia è **once-per-period** (`warningPeriodStart`) — evita allarmi ripetuti ogni richiesta.

### 2.6 `degradation.ts` — Degrado controllato (Full → Reduced → Minimal → Safe Default)

**Cosa fa.** Framework `withDegradation(feature, primary, fallback, safeDefault, options)`: tenta il percorso primario, poi il fallback, poi restituisce il safe default. Ogni transizione aggiorna un *registry globale* (per dashboard) con `level`, `reason`, `since` e `capability` (descrizione umana di cosa funziona ancora). `getDegradationReport` ordina per gravità; `hasAnyDegradation`, `getDegradationSummary`. Versione sync e async.

**Pattern architetturali.** *Fallback chain a 3 livelli*; *registry osservabile* (stato globale consultabile); *fail-safe con default permissivo* (es. rate limiting degradato → `{allowed: true, remaining: Infinity}`).

**Punti di forza / lezione per Nexus.** Il pattern è il più importante per un gateway che dipende da N servizi esterni: **ogni feature dovrebbe sapere cosa farà se il suo backend muore**. La granularità del "cosa offre ancora" (capability) è ciò che manca nella maggior parte dei degradi ad hoc. Nexus: adottare `withDegradation` per Redis, vector store, pagamento, telemetria — e montare il report su una dashboard /health.

### 2.7 `quotaCache.ts` — Cache delle quote provider (684 righe)

**Cosa fa.** Cache in-memory delle quote per provider-connection (percentuale rimasta per finestra temporale: 5h, weekly, per-modello). Popolata da due fonti: endpoint di usage e **risposte 429** (`markAccountExhaustedFrom429`). Background refresh ogni 1 minuto con batch di max 5 concurrency (anti thundering herd). Punti di forza specifici:
- **T08 auto-advance**: se `resetAt` è già passato, la quota è considerata disponibile subito, senza aspettare il refresh → nessun blocco falso con quota rinnovata.
- **TTL differenziati**: account attivi vs esauriti (5 min), e gli esauriti *senza* resetAt scadono dopo TTL fisso.
- **#4438 anti-write**: gli snapshot DB vengono salvati solo quando la quota **cambia davvero** (`quotaSnapshotChanged`), altrimenti 400K righe identiche al giorno.
- **Per-modello**: per Antigravity (`getAntigravityQuotaFamily`) e Codex (`getCodexQuotaWindowFilterForModel`) l'esaurimento è valutato **per finestra del modello richiesto** (famiglie gemini/claude aggregate), non a livello connessione.
- **#8065 critico**: lo stato vive su `globalThis` (`__omnirouteQuotaCacheState`), non in module-scope — perché i build Next.js `standalone` caricano il modulo in chunk separati con Map duplicate. Lezione architetturale fondamentale.
- **Hydration da snapshot DB**: se la cache è fredda, ricostruisce l'entry dalle ultime snapshot (`hydrateQuotaCacheFromSnapshots`) → restart senza buco di conoscenza.

**Pattern architetturali.** *Write-through con snapshot conditionale*; *lazy reset*; *stato globale condiviso* (globalThis pattern); *TTL a due velocità*; *per-model scoping*; *probing lazy*.

**Punti di forza / lezione per Nexus.** È il pezzo più maturo del progetto. Cinque lezioni:
1. **globalThis per lo stato** quando l'app può essere caricata in più chunk/processi — Nexus deve decidere subito se la quota cache è per-processo (fine) o condivisa (Redis).
2. Il 429 del provider è un **evento di stato** (marca esaurito), non solo un errore da ritentare.
3. L'auto-advance su `resetAt` elimina la classe di bug "ho appena sbloccato ma vengo ancora bloccato per 5 minuti".
4. Mai scrivere snapshot invariati: il diff è gratis e salva il DB.
5. Le quote per-modello (Antigravity/Codex) mostrano che un gateway deve sapere che **la quota non è per provider ma per modello/famiglia**.

### 2.8 `providerExpiration.ts` — Scadenza proattiva delle credenziali

**Cosa fa.** Traccia date di scadenza per `oauth_token`, `subscription`, `api_credits`, `free_tier_reset`. Calcola lo status (`active`/`expiring_soon`/`expired`/`unknown`) con `alertDays` configurabile (default 7). Espone summary, sort per gravità, e **`detectExpirationFromResponse`**: deriva la scadenza dagli header HTTP delle risposte (401→token scaduto, 402→subscription scaduta, 429 con `x-ratelimit-reset`/`retry-after`→free-tier reset, gestendo sia epoch-seconds che seconds-from-now).

**Pattern architetturali.** *Predizione da side-channel* (gli errori HTTP diventano segnali di stato); *stato derivato ricalcolato ad ogni lettura* (niente cron per aggiornare lo status).

**Punti di forza / lezione per Nexus.** I 401/402 sono **diagnosi** (token morto, abbonamento morto), non solo errori da ritentare. Nexus: ogni errore upstream deve alimentare un registro di scadenza credenziali e un alert "re-autentica ora", prima che le richieste inizino a fallire in massa. Il pattern "ricalcola lo status alla lettura" elimina i job di aggiornamento.

### 2.9 `modelAvailability.ts` — Report di indisponibilità modelli

**Cosa fa.** Fassade sottile su `accountFallback` (open-sse): `getAvailabilityReport` (provider, model, connectionId, reason, remainingMs, failureCount), `clearModelUnavailability`, `resetAllAvailability`.

**Pattern architetturali.** *Anti-corruption layer* / *facade*: il dominio non conosce l'implementazione dei lockout, solo il report.

**Punti di forza / lezione per Nexus.** Concetto: **la lista dei modelli bloccati è un report pubblico del gateway** (dashboard, debugging, retry da parte dei client). I lockout per-modello con `remainingMs` e `failureCount` sono lo stato che Nexus deve esporre in /health.

### 2.10 `connectionModelRules.ts` — Esclusioni modelli per connessione

**Cosa fa.** Regole per-connection: pattern wildcard (`excludedModels`) per escludere modelli da una connessione specifica. Normalizza i pattern (csv/array, dedup, `**`→null=nessun filtro), genera *candidati* di match per un modelId (il modello nudo senza prefisso provider, senza suffisso `[1m]`), e valuta il match wildcard. `hasEligibleConnectionForModel` verifica che esista almeno una connessione non-escludente per un modello.

**Pattern architetturali.** *Normalizzazione multi-formato* (stesso fronte delle altre utility di dominio); *match con candidati multipli* (robustezza sui naming dei modelli).

**Punti di forza / lezione per Nexus.** Prima di dire "nessun provider serve questo modello", il gateway deve chiedere "**c'è una connessione che NON esclude questo modello?**". Nexus: esclusioni per-connessione + candidati di matching normalizzati (strip prefix/suffix) = meno falsi negativi.

### 2.11 `pipeline.ts` — Auto-pipeline multi-stadio (plan → execute → reflect → fix)

**Cosa fa.** Motore puro di pipeline LLM multi-stadio, **senza side-effect**: l'esecuzione è delegata a una `StageExecutor` iniettata dal chiamante. Template per task type (`code`, `math`, `reasoning`, `creative`, `medium`, `simple`) con tier di fitness per stadio (es. code: plan=best-reasoning, execute=cheapest, reflect=moderate, fix=cheapest). Lo stadio **reflect** produce JSON strutturato (`parseReflectJson`: estrazione da code block/oggetto, conservative fail); se `pass` salta `fix`, se `fail` usa la versione corretta. Il contesto viene threadato tra gli stadi (plan_context → execution_response → reflection_response). La selezione dell'output migliore: fix > reflect-corrected > execute > ultimo riuscito. Qualsiasi errore di stadio → `fallback: true` e si ritorna il meglio disponibile.

**Pattern architetturali.**
- *Pipeline con dependency injection* (l'engine non tocca la rete — solo `StageExecutor`).
- *Reflection come gate di qualità* (LLM che valuta LLM, con contratto JSON rigido).
- *Fitness tier* come hint di selezione provider per stadio (costo/reasoning diversi per fase).
- *Best-available-output*: mai restituire niente pur di completare.

**Punti di forza / lezione per Nexus.** È il pattern "agentic" più concreto del progetto: **un gateway può eseguire catene multi-call con modelli diversi per fase** (pianificatore premium, esecutore economico, revisore medio). La lezione più trasferibile: il contratto JSON del reflect (pass/fail/issues/corrected) è un protocollo stabile e il **parse failure è trattato come fail** (conservative), mai come pass.

### 2.12 `assessment/assessor.ts` — Probe proattivi dei modelli

**Cosa fa.** `Assessor` invia richieste probe (livelli quick/standard/deep con payload e max_tokens diversi) attraverso il proxy verso `providerId/modelId`, con AbortController e timeout. Classifica l'esito (`working`, `broken`, `rate_limited`, `timeout`, `auth_error`), misura latenza P50/P95, calcola success rate, traccia `consecutiveFails`/`probeCount`, e produce `ModelAssessment` completo. `runAssessment` esegue una campagna su più modelli con trigger (scheduled/on_demand/on_error/startup).

**Pattern architetturali.** *Probe-based health* (sondaggio attivo, non solo osservazione passiva); *probe escalation* (quick → standard, si ferma su auth_error/broken per non sprecare richieste); *metriche percentili*.

**Punti di forza / lezione per Nexus.** Il gateway che **sonda proattivamente** i propri provider conosce il proprio catalogo meglio del provider stesso. La distinzione `auth_error` (credenziali, non ritentabile) vs `rate_limited` (transitorio, ritentabile) vs `broken` (permanente) è la tassonomia che Nexus deve adottare per il triage degli errori. Il probe standard con `max_tokens` minimo costa quasi nulla.

### 2.13 `assessment/categorizer.ts` — Classificazione e fitness dei modelli

**Cosa fa.** Assegna categorie (`coding`, `reasoning`, `reasoning_deep`, `chat`, `fast`, `vision`, `tool_call`, `structured_output`), tier (`premium`/`balanced`/`fast`/`free`) e **fitness score per categoria** tramite matrice di pesi (`CATEGORY_WEIGHTS`: ogni categoria pondera diversamente tier/speed/success/cost). Matching euristico per nome modello (regex su opus/sonnet/haiku/gpt/deepseek/gemma/glm/qwen/mini...), più segnali misurati (latenza <2s → fast, vision/tool_call/structured da probe).

**Pattern architetturali.** *Scorecard multi-criterio* (fitness = weighted sum normalizzata); *euristiche + misurazioni* combinate; *ontologia di capacità*.

**Punti di forza / lezione per Nexus.** La **fitness per categoria** è il motore del routing "giusto": per un task coding, il modello viene valutato con pesi coding (tier 40%, speed 30%, success 20%, cost 10%), non con un punteggio globale. Nexus: il selettore modelli dovrebbe ricevere la *categoria* del task e scegliere con la scorecard di quella categoria.

### 2.14 `assessment/selfHealer.ts` — Auto-riparazione dei combo

**Cosa fa.** `healCombo(combo, assessments)`: per ogni modello del combo guarda l'assessment → `broken` viene **rimosso**, `rate_limited`/`timeout` vengono **de-pesati** (`weight * (1 - maxWeightReduction)`, floor `minimumWeight`), sani restano. Se tutti i modelli muoiono → **emergency_replace** con il miglior working model dal template (per categoria+tier+fitness). Calcola `ComboHealth` (healthScore, autoFixCount) e logga ogni azione (`HealAction`). `generateCombosFromAssessments` auto-genera combo dai template (`AUTO_COMBO_TEMPLATES`: best-coding, best-reasoning, best-fast, pro-*...) selezionando i top-5 modelli con pesi decrescenti (35%/65% split).

**Pattern architetturali.** *Closed-loop control* (misura → azione → stato → nuova misura); *azioni graduate* (rimuovi / de-pesa / ripristina / sostituisci); *template-driven generation*.

**Punti di forza / lezione per Nexus.** È la più ambiziosa: **il gateway ripara da solo il proprio catalogo**. La scala di azioni è ben dosata (non si butta via un modello per un 429, lo si de-pesa). Le soglie (`brokenThreshold: 3`, `restoreThreshold: 2`, `maxWeightReduction: 0.5`, `minimumWeight: 5%`) sono sane. Nexus: iniziare dal de-pesaggio automatico (safe), arrivare alla rimozione solo dopo N fallimenti consecutivi.

### 2.15 `assessment/types.ts` — Contratto dati dell'engine di assessment

**Cosa fa.** Tutti i tipi del sotto-sistema: `AssessmentStatus`, `ModelCategory`, `ModelTier`, `ProbeLevel`, `AssessmentScope`, `ModelAssessment`, `AssessmentRun`, `AssessmentTrigger`, `ComboHealth`, `HealAction`, `AssessmentConfig`, `AUTO_COMBO_TEMPLATES`, `PROBE_MESSAGES`, `PROBE_MAX_TOKENS`, e `DEFAULT_ASSESSMENT_CONFIG` (quick 5min, standard 30min, deep 6h, timeout 30s, broken 3, restore 2).

**Pattern architetturali.** *Contract-first design*; *config centralizzata con default*; *template come dati*.

**Punti di forza / lezione per Nexus.** Le costanti di default documentate (intervalli probe, soglie) sono la "politica operativa" del gateway — Nexus deve tenerle in un unico posto revisionabile, non sparse.

### 2.16 `assessment/migration.ts` — Schema SQLite dell'assessment

**Cosa fa.** DDL idempotente (`CREATE TABLE IF NOT EXISTS`) per `model_assessments`, `assessment_runs`, `combo_health`, `heal_actions` con indici su status/provider/tier/last_tested/health_score. `runAssessmentMigration` abilita WAL, foreign keys e registra la migrazione in `_omniroute_migrations`.

**Pattern architetturali.** *Migration idempotente*; *WAL*; *registro versioni migrazioni*.

**Punti di forza / lezione per Nexus.** Le migrazioni DB sono parte del deploy, non una decisione a runtime. Indicizzare status/tier/last_tested è la chiave per query di assessment veloci su milioni di righe.

### 2.17 `persistence/comboRepositories.ts` — Contratti repository (port)

**Cosa fa.** Interfacce pure (JSDoc/TS): `ComboRepository` (list/count/findById/findByName/findByNameInsensitive/create/update/reorder/deleteById) e `ModelComboMappingRepository` (CRUD + `resolveForModel(model)` per la risoluzione pattern→combo). Niente implementazione.

**Pattern architetturali.** *Ports & Adapters* (hexagonal): il dominio definisce i port, l'infrastruttura li implementa.

**Punti di forza / lezione per Nexus.** È il segno più chiaro dell'architettura pulita: **il dominio dichiara il contratto di persistenza e non conosce il DB**. `resolveForModel` (mapping pattern→combo con priorità) è il meccanismo che Nexus deve usare per il "quale combo serve questo modello?".

### 2.18 `responses.ts` — Factory di risposte HTTP standard

**Cosa fa.** Helper puri: `successResponse`, `apiErrorResponse`, `badRequest`, `unauthorized`, `forbidden`, `notFound`, `conflict`, `tooManyRequests` (con header `Retry-After`), `internalError`. Formato errore uniforme `{error: {status, code, message, details}}`.

**Pattern architetturali.** *Response factory* / *convention over configuration*: un solo posto dove nascono le risposte → consistenza del contratto API.

**Punti di forza / lezione per Nexus.** Il formato di errore con **codice stabile** (`INVALID_INPUT`, `UNAUTHORIZED`, `RATE_LIMITED`...) è ciò che permette ai client di fare retry intelligenti. Nexus: adottare il formato `{error:{status,code,message}}` come unica forma di errore e il codice come contratto semantico.

### 2.19 `omnirouteResponseMeta.ts` — Header di telemetria `X-OmniRoute-*`

**Cosa fa.** Costruisce gli header di metadati su ogni risposta: `X-OmniRoute-Cache-Hit`, `-Latency-Ms`, `-Response-Cost`, `-Tokens-In/Out`, `-Version`, `-Model`, `-Request-Id`, `-Provider`, `-Cost-Saved` (solo su cache hit: cost evitato per il billing), `-Fallback-Attempts`, `-Decision` (`strategy=...; provider=...; latency_ms=...`). Sanitizzazione degli header: rimozione control-char, escaping non-ASCII via `encodeURIComponent` (Hard Rule #12: un header non deve mai veicolare secret/stacktrace). `attachOmniRouteMetaToResponse` muta gli header o clona la Response se immutabili (audio stream, passthrough).

**Pattern architetturali.** *Choke-point unico per telemetria* (ogni route deve passare da qui); *normalizzazione robusta dei valori* (`toNonNegativeInteger`, `toWellFormedUnicode`); *header condizionali* (omessi se non pertinenti).

**Punti di forza / lezione per Nexus.** Il concetto di **header di decisione di routing** (`strategy`, `provider`, `latency`, `fallback_attempts`) è la tracciabilità completa end-to-end: ogni risposta dice *chi* ha risposto e *perché*. Il `Cost-Saved` su cache hit separa correttamente billing (non addebitare) da analytics (mostrare il risparmio). Nexus: obbligare ogni route a passare da un unico costruttore di meta-header è la garanzia che nessuna risposta perda la telemetria.

### 2.20 `configAudit.ts` — Audit trail della configurazione

**Cosa fa.** Registra ogni modifica a provider/combo/policy/connection/settings con **snapshot before/after** e `computeDiff` (added/removed/changed, confronto via `JSON.stringify`), fonte del cambiamento (`dashboard`/`api`/`sync`/`auto-healing`/`cli`/`mcp`), filtri di ricerca e paginazione, `getRollbackState(entryId)` (restituisce lo stato precedente → rollback), `createSnapshot` (export completo con versione), summary per target/azione/fonte. Log limitato a 1000 entry in memoria.

**Pattern architetturali.** *Event sourcing leggero* (log append-only di cambi di stato); *rollback via before-snapshot*; *bounded log*.

**Punti di forza / lezione per Nexus.** Il rollback della configurazione è un requisito di sicurezza del gateway: se un auto-healer sbaglia, **si deve poter tornare indietro**. La distinzione della *sorgente* (auto-healing vs umano) è oro per il debugging. Nexus: log append-only con before/after + diff, bound in memoria e flush a DB.

### 2.21 `lockoutPolicy.ts` — Blocco anti-abuso per identificatore

**Cosa fa.** Traccia tentativi falliti per identificatore (IP, username, API key): finestra scorrevole (`attemptWindowMs` 5min), soglia (`maxAttempts` 5), durata lockout (15min), persistenza SQLite + cache in memoria. `checkLockout`, `recordFailedAttempt`, `recordSuccess` (pulisce), `forceUnlock`, `getLockedIdentifiers`. Il lockout scaduto viene azzerato lazy alla lettura.

**Pattern architetturali.** *Sliding window counter*; *persistenza con lazy cleanup*; *config con default*.

**Punti di forza / lezione per Nexus.** Il lockout anti-bruteforce deve stare nel policy engine (fase 1 del verdetto). Il lazy reset alla lettura (niente cron) è lo stesso pattern vincente del resto del codebase.

### 2.22 `prompts.ts` — Template prompt per la pipeline

**Cosa fa.** Template system/user per gli stadi `plan`/`execute`/`reflect`/`fix` con interpolazione `{variable}`. Il reflect impone un **formato JSON rigoroso** nel system prompt (pass/fail/issues/corrected).

**Pattern architetturali.** *Template + interpolazione*; *contratto di output nel prompt* (structured output via istruzione, senza schema binding).

**Punti di forza / lezione per Nexus.** La richiesta di JSON **esplicita e ripetuta** nel system prompt + parse robusto (code-block o oggetto) è il pattern pragmatico per ottenere structured output senza dipendere dal supporto `response_format` del provider.

### 2.23 `types.ts` — Tipi di dominio centralizzati (JSDoc)

**Cosa fa.** Definizioni centrali: `ProviderConnection` (con rateLimitOverrides rpm/tpm/tpd/minTime/maxConcurrent), `Combo`, `UsageEntry`, `ChatRequest`, `SanitizeResult`, `SecretsValidationResult`, `ProxyConfig`, `AppSettings`, `ApiError`. **Solo JSDoc, `export {}`** — nessun output TS.

**Pattern architetturali.** *Single source of truth per i tipi*; *JSDoc-as-types* (zero build step).

**Punti di forza / lezione per Nexus.** La scelta di usare JSDoc per i tipi di dominio (senza compilazione TS obbligatoria) è pragmatica per un codebase enorme: il contratto è documentazione leggibile e type-check opzionale. I `rateLimitOverrides` per-connessione sono il modello di configurazione che Nexus deve avere.

---

## 3. MECCANISMI PRINCIPALI `src/lib/`

### 3.1 Quota Sharing Engine — `src/lib/quota/`

**`enforce.ts`** — il gate PRE-request e il tracciatore POST-response della quota. `enforceQuotaShare(input)` → `EnforceDecision` (`allow`/`block` con `retryAfterSeconds`). Flusso: (1) trova i pool dell'API key, (2) filtra il pool della connessione (membership array `connectionIds`, D2), (3) risolve il piano provider (`resolvePlan`), (3b) **model-cap per (key, model)** con bucket separato `poolId:model:<model>`, (4) per ogni dimensione legge consumo (`store.peek`) e saturazione globale (`getSaturation`), (5) applica **fair-share** (`decideFairShare`). Lezione chiave: **fail-open obbligatorio (B16)** — ogni errore di store/piano/saturazione diventa `allow`, così un guasto di quota non blocca il traffico. I webhook `quota.exceeded` sono fire-and-forget. **`recordConsumption`** è asincrono post-risposta e ingoia gli errori (B29).

**`accountBuckets.ts`** — bucket di saturazione per (connectionId, windowKey). **Lazy reset**: `isBucketSaturated` cancella l'entry se `now >= resetsAtMs` — niente cron, il reset è *probing sul read path*. `updateAccountBuckets` normalizza le finestre Claude ("session (5h)"→5h, "weekly (7d)"→7d, "weekly <model> (7d)"→`7d:<model>`). Tempo iniettato (`nowMs`) per test deterministici.

**`fairShare.ts`** — algoritmo **work-conserving** a 3 policy (`hard`/`soft`/`burst`). In modo *generous* (saturazione < soglia) si può prendere in prestito capacità non allocata; in modo *strict* (>= soglia) si impongono le quote. Il **cap assoluto** è intrasgressibile sempre. **Normalizzazione policy fail-safe**: una policy corrotta/ignota viene trattata come `hard` (il più restrittivo) per chiudere la falla fail-OPEN (prima un'unknown policy cascava in `allow` silenzioso). Decisione a multi-dimensione.

**`planRegistry.ts` + `planResolver.ts`** — catalogo dei piani per provider (codex/claude: percent 5h+weekly; glm/minimax/kimi-coding: tokens con `limit=EPSILON` = "configura manualmente"; deepseek: usd monthly; kimi: requests hourly; grok-cli: requests/tokens daily+weekly). `resolvePlan` ha precedenza: **override DB manuale → catalogo → piano vuoto**. Lezione: la conoscenza di quanto vale ogni provider è *catalogata* e sovrascrivibile.

**`saturationSignals.ts`** — segnali di saturazione globale (0..1) per provider/dimensione, con **cache in-memory TTL 30s** perché l'endpoint `oauth/usage` upstream è rate-limited e 429 sotto carico (mai chiamarlo sul hot path senza cache). Fonti: fetcher dedicati per codex/bailian, `anthropic-ratelimit-*-utilization` per Claude, header `x-ratelimit-*` per la gestione proattiva. Fallback fail-open → 0 (generous).

**`sqliteQuotaStore.ts` / `redisQuotaStore.ts` / `storeFactory.ts`** — implementazioni del port `QuotaStore` (`consume`/`peek`/`poolConsumedTotal`/`poolUsage`/`clear`) con factory che sceglie il backend. Lezione: l'engine di quota è scritto contro un'interfaccia, la store è intercambiabile (Redis per multi-istanza).

**Lezione sintetica per Nexus.** Il sistema di quota è il più sofisticato: pool multi-account, fair-share work-conserving, policy hard/soft/burst, model-caps per (key,model), segnali di saturazione da upstream. I tre principi da rubare: **(1) fail-open strutturale, (2) proiezione a costo stimato, (3) separazione pre-check (enforce) / post-track (record)**.

### 3.2 `cacheLayer.ts` — LRU generica con limiti byte

**Cosa fa.** `LRUCache` in-memory con `maxSize` (default 50) e **`maxBytes`** (default 2MB) — doppio vincolo, evict più vecchio per inserimento. Chiave = SHA-256 dei params ordinati (dedup semantico). TTL per entry, statistiche (hits/misses/evictions/hitRate). Singleton `getPromptCache` configurabile via env.

**Pattern architetturali.** *LRU con doppio budget* (conteggio + byte); *content-hash key*; *statistiche embedded*.

**Lezione per Nexus.** Il limite in **byte** (non solo in entry) è ciò che salva un processo Node da un prompt da 1MB. La stima dimensione (`JSON.stringify(...).length * 2`) è un'euristica accettabile.

### 3.3 `semanticCache.ts` — Cache semantica due livelli (memoria + SQLite)

**Cosa fa.** Cache di risposte LLM **deterministiche**: chiave = SHA-256(model + messages normalizzati + temperature + top_p). Due livelli: LRU in memoria + tabella SQLite (sopravvive al restart, il hit dal DB viene **promosso in memoria**). **Isolamento per chiave API**: `apiKeyId` come prefisso PLAINTEXT della firma (non nel digest, per non rompere la determinismo né scatenare falsi positivi CodeQL). Metriche (hits/misses/tokens_saved) in tabella. Regole di cacheabilità: **richiede `temperature: 0` esplicito** (`isCacheableForRead/Write`) e rispetta `X-OmniRoute-No-Cache: true`. Invalida per modello, firma, o età.

**Pattern architetturali.** *Two-tier cache* (memoria calda + DB freddo); *promotion on read*; *cache key deterministico con namespace*; *policy di cacheabilità esplicita*; *billing-aware* (tokens_saved e header Cost-Saved).

**Lezione per Nexus.** Tre dettagli che fanno la differenza: **(1)** solo `temperature: 0` esplicito è cacheable (mai fidarsi del default del provider), **(2)** il namespace per API key impedisce hit cross-utente, **(3)** la metrica `tokens_saved` rende la cache monetizzabile/giustificabile. Lo streaming è cacheable (assemblato e servito come JSON).

### 3.4 `idempotencyLayer.ts` — Dedup idempotente su finestra breve

**Cosa fa.** Se una richiesta arriva con lo stesso `Idempotency-Key` o `X-Request-Id` entro 5s (configurabile), restituisce la risposta cachata invece di inoltrare di nuovo al provider. Cleanup periodico ogni 30s con timer `unref()`.

**Pattern architetturali.** *In-memory store con TTL*; *finestra breve* (5s) perché il problema è il double-submit, non la semantica di stato.

**Lezione per Nexus.** Il double-submit di un client (timeout + retry) può raddoppiare il costo LLM. Una finestra idempotenza di pochi secondi per (client, request) è il minimo sindacabile per un gateway.

### 3.5 `contextWindowResolver.ts` — Reconciler auto-correttivo delle finestre di contesto

**Cosa fa.** Feature 5004: confronta la finestra di contesto dichiarata dal provider (discovery) con quella del catalogo e, se divergono, **pinna il valore scoperto** come override `auto:discovery`. Non tocca mai gli override `manual`. Se il catalogo "recupera" (ora combacia), **rimuove l'override ridondante** → si auto-guarisce senza oscillare (mai confrontare con un valore già sovrascritto). Job periodico 24h, idempotente, fail-silent.

**Pattern architetturali.** *Reconciler puro con dependency injection* (la funzione `reconcileContextWindows` è testabile con stub); *self-healing data*; *sorgente di override con precedenza*.

**Lezione per Nexus.** I dati di catalogo (modelli, finestre, prezzi) **marciscono**; un job di reconcile che li corregge con l'osservazione reale, rispettando le sovrascritture manuali, è essenziale. Il commento anti-oscillazione (non confrontare con il valore che stai per scrivere) è una lezione di bug-design.

### 3.6 `freeProviderRankings.ts` — Ranking dei provider free per ELO

**Cosa fa.** Unisce i provider free (noauth/oauth/apikey) con i loro modelli (registry + custom) e gli **score ELO** della tabella `model_intelligence` (fonte Arena). Matching flessibile tra ID registry e nomi normalizzati (exact → strip version suffix → prefix match). Filtri `configuredOnly`/`availableOnly` con `isProviderUsable` (stato terminale `credits_exhausted`/`banned`/`expired` + rate-limited con scadenza). Funzioni pure esportate per test senza DB.

**Pattern architetturali.** *Join logico su dati eterogenei con matching fuzzy*; *ranking score-driven*; *predicati puri* (time iniettato).

**Lezione per Nexus.** Per la selezione dei provider free, un ranking basato su **intelligence reale (ELO)** con il filtro "usabile adesso" (non terminale, non rate-limited) è il modo giusto di ordinare le alternative economiche. Nexus: la classifica dei provider deve consumare lo stesso stato di health usato dal router (non una lista statica).

### 3.7 Compression engine — `open-sse/services/compression/` (RTK + Caveman)

**`caveman.ts` + `cavemanRules.ts`** — compressore a **regole**: ~50 classi di pattern testuali (pleonasmi, cortesie, hedging, filler, passive voice, intenzioni ridondanti, abbreviazioni ultra per configurazione/funzione...) con intensità `lite/full/ultra`. Preserva i blocchi da non toccare (`preservation.ts`: code, JSON) e ripristina dopo la compressione. Stima token e validazione.

**`adaptiveCompression/`** — il motore intelligente: `ladder.ts` definisce una **scala di escalation** di motori dal più lossless al più aggressivo (session-dedup → rtk → headroom → lite → caveman → aggressive → ultra → omniglyph), ognuno con `aggressiveness` e **fattore di riduzione atteso** (senza dry-run sul hot path). `resolveAdaptivePlan.ts` calcola il target di token dal contesto modello, e se il prompt stimato **non ci sta**, fa escalare la scala *partendo da sopra* il piano base (floor mode), fino a rientrare; se non si rientra mai, restituisce best-effort con `fit: false` (mai perdere contenuto per forzare il fit). "Già dentro il budget → mai over-compress".

**Lezione per Nexus.** La compressione è: **(1)** regole deterministiche a costo zero (Caveman), **(2)** più motori ordinati per aggressività con **fattori di riduzione stimati** (nessuna dry-run per richiesta), **(3)** **risoluzione adattiva che scala solo quando serve** e non distrugge contenuto. Il concetto di "ladder con floor (non ripartire da zero, scala oltre il piano base)" e di "telemetria di fit/headroom" è direttamente trasferibile a Nexus.

---

## 4. `src/server/` — STRATO MIDDLEWARE/AUTHZ

### 4.1 `authz/pipeline.ts` — Pipeline middleware completa

**Cosa fa.** `runAuthzPipeline` incapsula ogni richiesta: genera `requestId` → classifica la rotta (`classifyRoute`: PUBLIC / CLIENT_API / MANAGEMENT) → gestisce draining (503 durante shutdown) → check body size su metodi non-GET → **stripa gli header trusted** (`AUTHZ_TRUSTED_HEADERS`, `PEER_IP_HEADER`, `VIA_PROXY_HEADER`) così il token per-processo non raggiunge mai i route handler → stampa `routeClass`/`requestId` → preflight OPTIONS con CORS → IP filter (whitelist/blacklist, loopback esente per non chiudersi fuori) → policy per classe di rotta (`publicPolicy`/`clientApiPolicy`/`managementPolicy`) → per MANAGEMENT mutazioni: **validazione origin del browser + fallback CSRF token** → stampa del subject (`kind`, `id`, `scopes`) negli header → `NextResponse.next()` con headers arricchiti. Inoltre: refresh automatico del JWT dashboard (sliding window), redirect a login, e gestione errori JWT stale (delete cookie).

**Pattern architetturali.** *Middleware pipeline unica*; *Route classification first* (tutta la sicurezza dipende dalla classe, non dal path); *policy per classe di rotta* (strategia); *header stripping e re-stamping* (zero-trust dei forward); *peer locality* da stamp firmato (anti-spoof del Host header); *fail-closed per cookie-authed*, *fail-open relativo per token-authed*.

**Lezione per Nexus.** Il pattern di maggior valore: **ogni classe di rotta ha una policy di sicurezza dedicata** (PUBLIC/CLIENT_API/MANAGEMENT), e gli header di identità sono **strippati e re-innestati** dal middleware — i handler downstream non possono essere ingannati da header client. Il gate `draining` (503 durante graceful shutdown) e il check body-size centralizzato sono requisiti di produzione che Nexus non deve dimenticare.

### 4.2 `authz/classify.ts` — Classificazione rotte con alias

**Cosa fa.** Normalizza pathname (aliasing: `/v1` → `/api/v1`, `/codex` → `/api/v1/responses`, `/v1/v1` → de-duplicazione, `/v1beta` → `/api/v1beta`) e classifica in `MANAGEMENT`/`CLIENT_API`/`PUBLIC` con motivo (`reason`) per debugging. 

**Pattern architetturali.** *Canonicalizzazione path* (aliases come contratto di compatibilità).

**Lezione per Nexus.** Gli alias compatibili (OpenAI `/v1`, Anthropic `/v1beta`, Codex `/responses`) sono un layer di "protocollo" che Nexus deve esporre per essere drop-in dei client esistenti.

### 4.3 `authz/context.ts` + `authz/types.ts` — Contratto policy

**Cosa fa.** `AuthOutcome = AuthDecision | AuthRejection` (allow con subject / reject con status+code+message); `RoutePolicy` come interfaccia `evaluate(ctx)`. Il verdetto di ogni policy è tipizzato e serializzabile.

**Pattern architetturali.** *Discriminated union* per gli esiti (exhaustive switch).

**Lezione per Nexus.** Il verdetto tipizzato (`allow`/`reject` con codice stabile) è il contratto che unifica tutto il middleware.

### 4.4 `cors/origins.ts` — Allowlist CORS centralizzata

**Cosa fa.** Fonte di verità unica per CORS: no wildcard di default; `CORS_ALLOW_ALL=true` opt-in (echo origin con `Vary: Origin`); allowlist da env + runtime (settings persistite); normalizzazione (lowercase, strip slash). **`applyCorsHeaders(response, request, relaxForTokenAuth)`**: per il surface token-authenticated (`/v1*`, `/v1beta*`, readonly public) può rilassare l'origine (echo o `*`) — sicuro perché quei client si autenticano con header che i browser non auto-allegano, e **mai** accoppiato con `Allow-Credentials`; per MANAGEMENT (cookie) resta fail-closed. Aggiunge `Vary: Accept-Encoding` per caches (RFC 9110).

**Pattern architetturali.** *Choke-point CORS unico*; *policy differenziata per classe di autenticazione*; *Vary corretto per cache*.

**Lezione per Nexus.** La distinzione **"cookie-authed (fail-closed) vs token-authed (relax consentito)"** è la chiave per non rompere i client browser/Electron senza aprire una falla CSRF. Il CORS è un unico posto, mai logica sparsa nei route.

### 4.5 `auth/loginGuard.ts`, `ws/liveServer.ts`, `origin/publicOrigin.ts`

- `loginGuard.ts`: guard per pagine che richiedono login (management).
- `ws/liveServer.ts`: server WebSocket per l'aggiornamento live della dashboard (allowlist `liveServerAllowList`).
- `origin/publicOrigin.ts`: validazione origin per mutazioni del browser (`OMNIROUTE_PUBLIC_BASE_URL`), abbinata al CSRF token di fallback in pipeline.

---

## 5. SINTESI ESECUTIVA — LE 10 ARCHITETTURE DA CUI NEXUS PUÒ IMPARARE

1. **Fallback dichiarativo + esclusione provider già tentati** (`fallbackPolicy` + `getComboFallbacks`). La catena di fallback è *dato* (tabella modello→lista ordinata), si risolve in un lookup, e `excludeProviders` impedisce i loop. Nexus: un modulo `fallbackPolicy` con cache-aside + persistenza e il concetto di exclude-set per request-cycle.

2. **Quota cache auto-riparante** (`quotaCache`). Stato su `globalThis` (anti chunk-split), TTL differenziati (attivo/esaurito), **auto-advance sul resetAt** (nessun blocco falso dopo il reset), snapshot DB **solo se cambiano** (anti-scritture inutili), hydration dal DB a cache fredda, e quote **per-modello/famiglia** (Antigravity/Codex). Nexus: è il modello di riferimento per la quota cache, incluse le tre feature anti-bug (globalThis, auto-advance, diff-on-write).

3. **Budget in proiezione + finestre di reset lazy** (`costRules`). Il controllo budget usa `periodUsed + additionalCost` (costo stimato della richiesta in arrivo); le finestre daily/weekly/monthly con ora di reset sono calcolate in UTC e il **rollover avviene lazy alla lettura** (nessun cron). Warning una-volta-per-periodo. Nexus: `checkBudget(key, additionalCost)` come API del gateway.

4. **Assessment proattivo + tassonomia errori a 5 classi** (`assessment/assessor`). Il gateway sonda i provider con probe a basso costo e distingue `working/broken/rate_limited/timeout/auth_error`. La tassonomia è la base del triage: auth_error non è ritentabile, rate_limited sì. Nexus: probe standard con timeout AbortController e metriche percentile.

5. **Self-healing graduato dei combo** (`assessment/selfHealer`). Nessun modello viene buttato per un 429: si **de-pesa**; solo dopo soglie di fallimento consecutivo si rimuove; se il combo muore, **emergency_replace** col miglior working model del template. Health score e log azioni. Nexus: closed-loop "misura → de-pesa → rimuovi → sostituisci" con soglie conservatrici.

6. **Fitness per categoria** (`assessment/categorizer`). Il punteggio di un modello è **specifico per task** (matrice pesi per coding/reasoning/chat/fast/vision...): la selezione del provider per un task coding usa la scorecard coding. Nexus: il router sceglie con la scorecard della categoria del task, non con un punteggio globale.

7. **Quota engine fair-share work-conserving** (`lib/quota`: enforce/fairShare/planRegistry/saturationSignals). Pool multi-account, quote per (key, model), policy hard/soft/burst, saturazione globale da upstream con cache 30s, **fail-open strutturale** (B16) e separazione pre-check/post-track (enforce/record). Nexus: il gate di quota deve fallire in aperto e proiettare a costo stimato.

8. **Cache deterministica a due livelli con billing-aware** (`semanticCache` + `cacheLayer`). Solo `temperature: 0` esplicito è cacheable, namespace per API key (isolamento cross-utente), memoria + SQLite con promotion, metriche `tokens_saved` e header `X-OmniRoute-Cost-Saved`. Nexus: due-tier cache con policy esplicita di cacheabilità e costo salvato esposto.

9. **Compressione adattiva a scala di motori** (RTK+Caveman ladder). Motori ordinati per aggressività con fattori di riduzione stimati (niente dry-run sul hot path); **si scala solo se il prompt stimato non rientra** e il floor parte sopra il piano base; mai over-compress se già nel budget; mai perdere contenuto (best-effort con `fit:false`). Nexus: ladder di compressori + resolver adattivo con telemetria fit/headroom.

10. **Middleware authz per classi di rotta + telemetria come choke-point** (`server/authz/pipeline` + `omnirouteResponseMeta`). Classificazione rotta (PUBLIC/CLIENT_API/MANAGEMENT) con policy dedicata, header trusted strippati e re-innestati, CORS fail-closed per cookie / relax per token, draining 503 in shutdown; ogni risposta porta header `X-OmniRoute-*` (strategy, provider, latency, fallback_attempts, cost, tokens) costruiti in un unico punto sanitizzato. Nexus: stesso dual pattern — sicurezza per classe di rotta e telemetria a choke-point unico.

---

## 6. RILEVAZIONI TRASVERSALI E RISCHI DA EVITARE IN NEXUS

- **Stato in module-scope** è un trap (`#8065`): con build multi-chunk o più istanze ogni modulo ha la sua Map. Decidere subito per-processo vs condiviso (globalThis pattern / Redis).
- **Counter in-memory** (round-robin combo, contatori idempotenza) non sopravvivono a restart/multi-istanza — accettabile solo per finestre brevi.
- **JSDoc senza TS** rende il refactoring più rischioso a scala 11k+ file: Nexus può adottare il contratti-tipi ma valutare TypeScript per i moduli critici (quota, fallback).
- **Import dinamici** nel dominio (webhookDispatcher, driverFactory) servono per il lazy-load e il boot pre-DB — pattern da replicare con cura (mai catene circolari).
- I **commenti-con-lezione** (numeri di issue #8065, #4438, #5923, Hard Rule #12, decisioni B16/B25/B29) sono eccezionalmente utili: Nexus dovrebbe mantenere la stessa disciplina di *decision log inline*.

*Fine rapporto. Analisi statica su 24 file di dominio + 3 sotto-sistemi lib + middleware server + engine compressione.*
