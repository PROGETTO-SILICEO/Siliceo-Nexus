# 🔒 Siliceo-Nexus — Documentazione Architetturale Interna

![Siliceo-Nexus Banner](assets/banner.png)

> **Documento Riservato al Progetto Siliceo — Uso Interno (Alfonso Riva & Nova Kernel)**  
> Mappa dell'infrastruttura di rete, integrazione dei microservizi e gestione dei nodi GPU.

---

## 🏛️ Topologia di Rete & Mappa Servizi Siliceo

```
                             ┌──────────────────────────────────┐
                             │       NODO OPERATIVO LOCALE      │
                             │        (Alfonso / Centro)        │
                             └──────────────────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────────────┐
              ▼                               ▼                               ▼
  ┌───────────────────────┐       ┌───────────────────────┐       ┌───────────────────────┐
  │   Nova Kernel v3      │       │     Memory Proxy      │       │  Siliceo-Nexus (Rust) │
  │   Porta HTTP: :3000   │       │   Porta HTTP: :3001   │       │   Porta HTTP: :8082   │
  └───────────────────────┘       └───────────────────────┘       └───────────────────────┘
              │                                                               │
              │                               ┌───────────────────────────────┘
              ▼                               ▼
 ┌────────────────────────────────────────────────────────┐
 │   MEMORY SERVER REMOTO (SQLite + Prisma)               │
 │   URL: http://100.114.216.76:3003                      │
 └────────────────────────────────────────────────────────┘
              │
              ▼ Tailscale Mesh
 ┌────────────────────────────────────────────────────────┐
 │   NODO INFERENZA GPU (RTX 2070) — IP 100.98.20.76        │
 │   - Beellama / llama.cpp (LLM Text): http://:8080      │
 │   - ComfyUI (Immagini/Vision):        http://:8188      │
 │   - Parakeet / Voice TTS:            http://:1135      │
 └────────────────────────────────────────────────────────┘
```

---

## ⚙️ Configurazione Interna dei Provider & Multi-Key Pool

Siliceo-Nexus (`:8082`) gestisce l'ottimizzazione dei costi ($0.00 USD) ed il failover resiliente secondo queste regole:

### 1. Cascade Priority & Intent Tags

| Provider | Priorità | Tier | Endpoint / Model | Tag Intent | Costo |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`beellama-tailscale-2070`** | **P1** | `local` | `http://100.98.20.76:8080`<br>`Qwen3.5-4B-Instruct-Q5_K_M.gguf` | `chitchat`, `fast`, `local`, `tool_supported` | **$0.00** |
| **`openrouter-llama-70b-free`** | **P2** | `free` | `https://openrouter.ai/api/v1`<br>`meta-llama/llama-3.3-70b-instruct:free` | `chitchat`, `reasoning`, `cloud_free` | **$0.00** |
| **`openrouter-qwen-coder-free`** | **P2** | `free` | `https://openrouter.ai/api/v1`<br>`qwen/qwen-2.5-coder-32b:free` | `coding`, `cloud_free` | **$0.00** |
| **`gemini-free-tier`** | **P3** | `free` | `https://generativelanguage.googleapis.com`<br>`gemini-2.5-flash` | `chitchat`, `coding`, `fast` | **$0.00** |

---

## 🔑 Multi-Key Pool & Round-Robin Key Rotation (`src/adapters.rs`)

Per evitare del tutto i Rate Limit sui provider gratuiti, la funzione `pick_api_key` supporta **chiavi multiple per ogni provider**.

Se nel campo `api_key` (o nelle variabili di ambiente `GEMINI_API_KEY`, `GROQ_API_KEY`, `OPENROUTER_API_KEY`) si inseriscono più chiavi separate da virgola (es. `KEY_ALPHA, KEY_BETA, KEY_GAMMA`), Nexus:
1. Ruota le chiavi in **Round-Robin** ad ogni richiesta.
2. Se una chiave riceve `HTTP 429`, la isola e tenta istantaneamente la chiave successiva del pool prima di attivare il failover del provider.

---

## 📦 Sincronizzazione Catalogo 394 Modelli OpenRouter (`src/catalog.rs`)

Il modulo `catalog.rs` esegue il sync automatico all'avvio e **ogni 24 ore** da `https://openrouter.ai/api/v1/models`.

- **Endpoint di Consultazione**: `GET http://localhost:8082/catalog`
- **Endpoint di Refresh a Caldo**: `POST http://localhost:8082/catalog/sync`
- **Dati censiti nel DB**: `model_id`, `prompt_cost_per_1m`, `completion_cost_per_1m`, `context_length`, `is_free`, `last_updated`.

---

## 🚀 Gestione Servizio Systemd

```bash
# Stato del servizio
systemctl --user status siliceo-nexus.service

# Riavvio a caldo
systemctl --user restart siliceo-nexus.service

# Visualizzazione Log Live
journalctl --user -u siliceo-nexus.service -f
```
