# 💎 Siliceo-Nexus

> **Universal Multimodal Inference Gateway & Task-Aware Mesh Engine**  
> Microservizio standalone in Rust (`:8082`) per l'orchestrazione dinamica dell'inferenza (Testo, Immagini, Voce) per l'intero ecosistema Siliceo (Nova, Silicea, Poeta, HKStyle, OpenCode, IDE ed estensioni).

---

## 🏛️ Cos'è Siliceo-Nexus

**Siliceo-Nexus** è l'infrastruttura d'inferenza unificata di rete. Agisce come punto di contatto unico (`http://localhost:8082/v1`) per qualsiasi programma o agente che necessita di chiamate LLM o multimodali, disaccoppiando l'intelligenza cognitivo-decisionale dal trasporto computazionale.

### Principi Architetturali Fondamentali

1. **Zero Modelli Hardcoded**: Nessun nome di modello scritto nel codice compilato. Tutto è basato su **Capability & Tag dinamici** caricati da database SQLite (`data/nexus.db`).
2. **Mesh Tailscale Native**: Connessione diretta col **`nodo-inferenza` (`100.98.20.76`)** via Tailscale per l'esecuzione su GPU RTX 2070 (beellama/Ollama `:8080`, ComfyUI `:8188`, Servizio Voce).
3. **Free-Tier Aggregation Pool**: Aggregazione a rotazione e stacking di provider gratuiti (Gemini, Groq, Cerebras, SambaNova, OpenRouter Free) per abbattere i costi a **$0.00 USD**.
4. **Controllo Utente Totale (Governance)**: Dashboard SPA su `http://localhost:8082/` per la gestione a caldo delle chiavi API (Free vs Paid), priorità e switch manuali.

---

## 🗺️ Topologia della Rete Siliceo Mesh

```
                                    ┌─────────────────────────────────────────────────────────────┐
                                    │ NODO INFERENZA TAILSCALE (100.98.20.76 - RTX 2070)          │
                                    │ ├─► beellama (:8080)   [Testo locale Qwen3.5/Coder 4B/9B]   │
                                    │ ├─► ComfyUI  (:8188)   [Generazione Immagini & Workflow]    │
                                    │ └─► Voce     (TTS/STT) [Sintesi Vocale & Trascrizione]       │
                                    └─────────────────────────────────────────────────────────────┘
                                                                 ▲
                                                                 │ (Via Tailscale Mesh)
┌────────────────────────────────┐                               │
│ CLIENTS NETWORK SILICEO:       │   HTTP Standard OpenAI        │
│ - Nova (Kernel v3 / v2)        │───────────────────────────────┼───► 💎 Siliceo-Nexus (:8082)
│ - Silicea / Poeta / HkStyle    │   (Porta :8082)               │     (Centro Operativo 100.79.151.73)
│ - OpenCode / Claude Code / IDE │                               │
│ - Script Python / Extension WS │                               │
└────────────────────────────────┘                               │
                                                                 ▼
                                    ┌─────────────────────────────────────────────────────────────┐
                                    │ FREE-TIER STACKING POOL (Cloud $0 USD)                      │
                                    │ - Gemini Free Keys Pool (15 RPM / TPM Tracking)             │
                                    │ - Groq Free Pool (Llama 3.3 70B fast)                       │
                                    │ - Cerebras Ultra-Fast / SambaNova / OpenRouter Free         │
                                    └─────────────────────────────────────────────────────────────┘
```

---

## 🛡️ Guardrail di Resilienza Tecnica (Anti-Failure)

Per garantire la stabilità in produzione, Siliceo-Nexus implementa 5 guardrail nativi:

### 1. Token Bucket al Minuto (TPM Tracking)
Oltre al conteggio delle richieste (RPM), Nexus traccia i **Token al Minuto (TPM)**. Se una chiave/provider raggiunge il tetto di token (es. 32.000 TPM su Gemini Free), la chiave viene messa in **cooldown preventivo** prima che scatti il ban `HTTP 429`.

### 2. Tag `tool_supported: true` per Cicli ReAct
I cicli cognitivi con Function Calling XML/JSON vengono instradati **esclusivamente** verso modelli e provider con tag `tool_supported: true`, impedendo a modelli free non addestrati di corrompere i cicli ReAct.

### 3. VRAM Swap Warmup Timeout (25s) per la RTX 2070
Quando il `nodo-inferenza` deve cambiare modello residente in VRAM (es. da chitchat a coder), Nexus estende il timeout a **25 secondi**, evitando il crash del client durante il caricamento del modello in GPU.

### 4. Protezione Anti-Ban IP (Multi-Provider Stacking)
Anziché abusare di più chiavi dello stesso provider sullo stesso IP, Nexus predilige il rotamento tra **provider diversi** (1 Gemini + 1 Groq + 1 Cerebras + 1 SambaNova), prevenendo il blocco anti-sybil degli account.

### 5. SQLite WAL Mode Native
La base dati `data/nexus.db` è configurata nativamente in **WAL Mode** (`PRAGMA journal_mode=WAL; busy_timeout=5000;`), garantendo letture ad altissima velocità durante gli inserimenti concorrenti e le sincronizzazioni del catalogo.

---

## 🧠 Smart Task Routing (Instradamento Dinamico per Contesto)

Nexus analizza al volo l'intento del messaggio senza aggiungere latenza:

- **Tier Chitchat / Presenza**: Saluti, conversazione diretta (`< 80` chars, "ciao", "come va") ➔ Smistato su `beellama` locale (2070) o Gemini Flash Free. Latenza bassissima, costo $0.
- **Tier Coding & Refactoring**: Snippet, file `.rs`, `.py`, traceback, keywords `fn`, `impl`, `def` ➔ Smistato su provider con tag `coding` (es. Qwen-Coder / DeepSeek v4).
- **Tier Reasoning & Audit**: Piani architetturali, evaluazioni del Tribunale ➔ Smistato su provider con tag `reasoning` (es. DeepSeek R1 / Sonnet).

Client Hints: `model: "auto:coder"`, `model: "auto:chat"`, `model: "auto:fast"`.

---

## 🎛️ Dashboard di Controllo (`http://localhost:8082/`)

Served direttamente dal binario Rust di Siliceo-Nexus:

1. **Multi-Key Manager**: Gestisci chiavi **Free vs Paid** per lo stesso provider (es. usa la chiave Paid Gemini solo se il pool Free è esaurito o per task marcati `CRITICAL`).
2. **Auto-Catalog Sync**: Visualizzazione del catalogo modelli con costi trasparenti per 1M token ($0.00 vs $0.15), context size e tag.
3. **Live Token Economy**: Monitoraggio dei token e del risparmio economico generato dal Free-Tier Stacking.

---

## 📡 API Surface (OpenAI Standard)

- `POST /v1/chat/completions` — Testo, Chat & ReAct
- `POST /v1/images/generations` — Immagini/Video (ComfyUI `:8188`)
- `POST /v1/audio/speech` — Sintesi Vocale (TTS)
- `GET /v1/models` — Catalogo modelli attivi nel Nexus
- `GET/POST /providers` — Management API a caldo

---

## 🛠️ Quick Start

```bash
cd Siliceo-Nexus
cargo build --release
./target/release/siliceo-nexus
# Dashboard disponibile su http://localhost:8082/
```

---

*Siliceo-Nexus — Il Prisma dell'Inferenza Universale.* 💎
