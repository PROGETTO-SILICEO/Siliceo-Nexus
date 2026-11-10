# Nexus — completamento e controllo completo dei flussi

**Data**: 3 novembre 2026 · **Stato**: piano operativo, non ancora implementato

## 1. Diagnosi

Nexus non è “rotto”: è **incompleto**.

Oggi sa fare bene una cosa:

```
client → Nexus :8082 → provider LLM (locale o cloud)
```

Non sa fare:

- gestire servizi GPU diversi dal LLM (ComfyUI, voce/STT/TTS);
- chiedere al nodo di caricare il modello giusto;
- arbitrare la VRAM tra servizi;
- esporre un catalogo di servizi e modelli locali;
- mostrare nel dashboard chi occupa la GPU e perché.

Il nodo, dal canto suo, ha tre pezzi separati:

- `beellama-switcher` :8080 → solo LLM;
- `gpu_monitor` :8282 → vede VRAM/processi e comanda ComfyUI;
- `ComfyUI` :8188 → servizio immagini;
- voce/Parakeet :1135 → previsto, non attivo;
- `homelab-agent` :8283 → telemetria.

Nessuno è il proprietario unico della GPU. La VRAM la vince chi arriva prima.

## 2. Architettura obiettivo

```
client / Sportello / agenti
        │
        ▼
   NEXUS :8082                  ← control plane: auth, routing, quote, log
        │
        ├─ /v1/chat/completions      → servizio llm
        ├─ /v1/images/generations    → servizio image
        ├─ /v1/audio/speech          → servizio voice
        ├─ /v1/audio/transcriptions  → servizio stt
        ├─ /v1/models                → catalogo aggregato
        └─ /v1/services              → stato nodi/servizi
        │
        ▼
   NODE AGENT :8080              ← data plane: lease GPU, start/stop, proxy
        │
        ├─ llama-server :8086    (9B, TurboQuant)
        ├─ ComfyUI :8188
        └─ voice :1135
```

**Regola:** nessun client pubblico parla con il nodo. Solo Nexus parla con il
Node Agent. Il Node Agent non decide il routing: esegue il lease e proxa.

## 3. Node Agent

Estensione dello switcher attuale, stessa porta `8080`, compatibilità con
`/v1/switch_model`.

### Endpoint

```http
GET  /health
GET  /v1/services
POST /v1/lease
POST /v1/release
POST /v1/switch_model        (compatibilità)
POST /v1/route               (proxy generico)
```

### `/v1/services`

```json
{
  "node": "nodo2070",
  "gpu": {"name": "RTX 2070", "total_mb": 8192, "used_mb": 3521, "free_mb": 4266},
  "active": {"service": "image", "model": "comfyui", "lease_id": "..."},
  "services": [
    {"id": "llm", "kind": "llm", "port": 8086, "running": false, "models": ["Qwen_Qwen3.5-9B-Q4_K_M.gguf"]},
    {"id": "image", "kind": "image", "port": 8188, "running": true, "models": ["comfyui"]},
    {"id": "voice", "kind": "voice", "port": 1135, "running": false, "models": ["parakeet"]},
    {"id": "stt", "kind": "stt", "port": 1135, "running": false, "models": ["parakeet"]}
  ]
}
```

### `/v1/lease`

```json
{"service": "llm", "model": "Qwen_Qwen3.5-9B-Q4_K_M.gguf", "ttl_s": 600}
```

Risposta:

```json
{"ok": true, "lease_id": "...", "service": "llm", "endpoint": "http://127.0.0.1:8086", "expires_at": "..."}
```

Il lease:

1. verifica che il servizio esista;
2. se un altro servizio tiene la GPU, chiede lo stop (solo servizi registrati);
3. attende che la VRAM scenda sotto soglia;
4. avvia il servizio;
5. attende l’health;
6. restituisce endpoint e lease;
7. dopo il TTL, scarica se inattivo.

Se un processo **non registrato** occupa VRAM (es. LFM3), il lease risponde
`409 conflict` con PID e nome. Non lo uccide alla cieca: serve un comando
esplicito o un allowlist.

### Modello come chiave

```
llm:Qwen_Qwen3.5-9B-Q4_K_M.gguf
image:comfyui
voice:parakeet
stt:parakeet
```

## 4. Nexus: cosa aggiungere

### 4.1 Tabelle

```sql
CREATE TABLE nodes (
  id TEXT PRIMARY KEY,
  base_url TEXT NOT NULL,
  auth_token TEXT,
  last_seen TEXT,
  enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE services (
  id TEXT PRIMARY KEY,
  node_id TEXT NOT NULL,
  kind TEXT NOT NULL,          -- llm | image | voice | stt
  name TEXT NOT NULL,
  base_url TEXT NOT NULL,
  models TEXT NOT NULL,        -- JSON array
  tags TEXT NOT NULL,          -- JSON array
  priority INTEGER NOT NULL DEFAULT 100,
  max_ctx INTEGER,
  enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE leases (
  id TEXT PRIMARY KEY,
  service_id TEXT NOT NULL,
  model TEXT,
  acquired_at TEXT NOT NULL,
  expires_at TEXT,
  status TEXT NOT NULL
);
```

### 4.2 Endpoint

| Endpoint | Funzione |
|---|---|
| `GET /v1/services` | stato nodi/servizi, VRAM, lease |
| `POST /v1/services/:id/lease` | acquisizione manuale |
| `POST /v1/services/:id/release` | rilascio |
| `POST /v1/images/generations` | OpenAI-compatible → ComfyUI |
| `POST /v1/audio/speech` | TTS |
| `POST /v1/audio/transcriptions` | STT |
| `POST /v1/route` | payload generico per workflow |
| `GET /v1/models` | aggregato LLM + image + voice |

### 4.3 Routing

Per ogni richiesta:

1. dedurre il servizio dall’endpoint o dal prefisso del modello;
2. scegliere il nodo/servizio per capability, priorità e salute;
3. chiamare `POST /v1/lease` sul Node Agent;
4. se `409` o `503`: coda breve, poi fallback cloud (solo LLM);
5. proxare la richiesta all’endpoint restituito;
6. registrare latenza, modello, provider, esito;
7. rilasciare o rinnovare il lease.

### 4.4 Dashboard

Nuova scheda **“Nodi & Servizi”**:

- GPU: VRAM totale/usata/libera, temperatura, util;
- servizio attivo e lease;
- modelli disponibili per servizio;
- pulsanti: carica 9B, avvia/ferma ComfyUI, avvia voce, rilascia GPU;
- richieste/min, latenza p95, error rate per servizio;
- ultimo cambio modello e motivo.

## 5. Voce

Nel README interno è previsto **Parakeet / Voice TTS :1135**, ma la porta non è
aperta e la voce attuale è **Piper locale su CPU** (`siliceo-daemon/voice`).
Prima di cablare la voce bisogna decidere:

- **A. Piper CPU locale** su `centro-operativo`: nessuna VRAM, nessun lease GPU;
- **B. Parakeet/TTS sul nodo**: servizio GPU, entra nel lease;
- **C. entrambi**: Piper per italiano leggero, Parakeet per qualità.

Consiglio: **A per la v1**, così la voce non compete con il 9B. B si aggiunge
dopo, con il suo lease.

## 6. Fasi

### Fase 0 — Ordine
- backup `nexus.db`, rotazione log (44 MB);
- mappa servizi (`docs/NODO_SERVIZI.md`);
- congelare porte e responsabilità.

### Fase 1 — LLM locale
- aggiornare provider a `Qwen_Qwen3.5-9B-Q4_K_M.gguf`, `max_ctx=8192`, tag `reasoning`;
- liberare VRAM (ComfyUI + LFM3) con permesso esplicito;
- precaricare 9B con TurboQuant;
- test chat attraverso Nexus.

### Fase 2 — Node Agent
- estendere switcher con `/v1/services`, `/v1/lease`, `/v1/release`;
- registry servizi e lease;
- test conflitto VRAM e idle timeout.

### Fase 3 — Nexus servizi
- tabelle `nodes/services/leases`;
- endpoint `/v1/services` e lease;
- dashboard “Nodi & Servizi”.

### Fase 4 — Image
- `/v1/images/generations` → ComfyUI;
- test prompt → immagine;
- stop/start ComfyUI tramite Node Agent.

### Fase 5 — Voice
- decidere A/B/C;
- `/v1/audio/speech` e `/v1/audio/transcriptions`;
- test end-to-end.

### Fase 6 — Cutover
- Sportello e sito usano solo Nexus;
- disabilitare accessi diretti al nodo;
- metriche, rate limit, allarmi.

## 7. Rischi

- **8 GB non bastano per LLM + ComfyUI insieme**: il lease è obbligatorio.
- **Processi non registrati** (LFM3) possono bloccare il 9B: serve un allowlist.
- **Nexus non è un gateway media**: va esteso, non basta un provider.
- **OpenAI image/audio** hanno formati diversi: servono adapter dedicati.
- **Voce**: Parakeet non è attivo; Piper è CPU e non ha un’API HTTP.
- **Log**: 44 MB e provider rumorosi; serve rotazione e pruning provider.

## 8. Criteri di accettazione

- [ ] LLM 9B risponde attraverso Nexus, non direttamente.
- [ ] ComfyUI genera un’immagine attraverso Nexus.
- [ ] La voce (Piper o Parakeet) risponde attraverso Nexus.
- [ ] Un solo lease GPU alla volta; conflitto → 409 con PID.
- [ ] Fallback cloud LLM se il nodo è occupato.
- [ ] Dashboard mostra servizio attivo, VRAM, lease, latenza.
- [ ] Nessun client pubblico conosce l’IP del nodo.
- [ ] Test di carico: 10 richieste concorrenti, coda o 429, nessun crash.
- [ ] Log ruotati; provider rotti disabilitati.

## 9. Terreno preparato (03/11/2026)

Senza toccare la 2070:

- branch `feat/nexus-services`;
- `docs/NODO_SERVIZI.md` — mappa reale del nodo;
- `docs/NEXUS_COMPLETAMENTO.md` — questo piano;
- `docs/node_agent_openapi.yaml` — contratto del Node Agent;
- `docs/schema_services.sql` — schema `nodes`/`services`/`leases`, seed disabilitato;
- `scripts/mock-node-agent.py` — finto Node Agent su `:18080`, testato per health, services, lease, conflitto 409, chat, image, audio;
- nessuna chiamata di scrittura al nodo, nessun modello caricato, nessun servizio fermato.

Prossimo passo quando la 2070 è libera: Fase 1 (provider 9B + test chat attraverso Nexus).
