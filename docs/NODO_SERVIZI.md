# Nodo 2070 — mappa dei servizi e router unico

**Data**: 3 novembre 2026 · **Stato**: mappatura, non ancora implementato

## 1. Fotografia reale

Nodo: `100.98.20.76` · RTX 2070 **8 GB** · RAM 12,4 GB · Ubuntu · Tailscale.

| Porta | Servizio | Stato | GPU | Note |
|---:|---|---|---|---|
| 8080 | `beellama-switcher-rust` | online | indiretta | `/health`: `active_model=none`, `internal_port=8086`, cache `turbo4`/`turbo3_tcq` |
| 8086 | `llama-server` interno | spento | sì | ascolta solo su `127.0.0.1` del nodo; non raggiungibile da Tailscale |
| 8188 | ComfyUI 0.33.1 | **running** | sì | `/system_stats` risponde; `argv`: `/mnt/disco1/comfyui/main.py --port 8188` |
| 8282 | `gpu_monitor.py` | online | no | `/stats`, `/comfy/status`, controllo VRAM/processi |
| 8283 | `homelab-agent` | online | no | telemetria CPU/RAM/dischi |
| 8443 | HTTPS con certificato LE | aperto | ? | da identificare |
| 8444 | HTTPS con certificato LE | aperto | ? | da identificare |
| — | Voce / Piper | non sul nodo | no | Piper è locale su `centro-operativo` (`siliceo-daemon/voice/bin/piper`), non è un servizio HTTP del nodo |

### VRAM al momento della misura

```
usata 3521 MB / 8192 MB · libera 4266 MB · util 32%
processi:
  1079  comfyui/venv/bin/python    96 MB
  598366 lfm3-venv/bin/python    3416 MB
llm:   spento (active_model none)
comfy: acceso
```

**Conseguenza:** il 9B Q4 (~6,2 GB) **non entra** adesso. Non è un problema del modello: mancano ~2 GB e la GPU è occupata da ComfyUI + un processo LFM3 non gestito dallo switcher.

## 2. I tre pezzi che non si parlano

1. **Switcher** (`:8080`) gestisce solo `llama-server`: sa caricare/scaricare un GGUF, non sa fermare ComfyUI, non sa cosa sia la voce, non conosce i processi LFM3.
2. **gpu_monitor** (`:8282`) vede la VRAM e i processi, e ha i comandi Comfy, ma non è il gateway delle richieste.
3. **Nexus** (`:8082`) instrada solo richieste LLM; non ha un concetto di “servizio” image/voice. Il provider `beellama-tailscale-2070` punta a un modello che il nodo non ha (`Qwen3.5-4B-Q6_K.gguf`), con `max_ctx=4096`, ed è al 100% di errore.

Risultato: nessuno dei tre è il proprietario unico della GPU. La VRAM la vince chi arriva prima.

## 3. Target: un Node Router, non tre pannelli

Un solo demone Rust sul nodo, sulla porta `8080`, che espone un contratto unico:

```http
GET  /health
GET  /v1/services
POST /v1/route
```

### `/v1/services`

```json
{
  "active": {"service": "llm", "model": "Qwen_Qwen3.5-9B-Q4_K_M.gguf"},
  "services": [
    {"id": "llm",   "port": 8086, "running": false, "models": ["Qwen_Qwen3.5-9B-Q4_K_M.gguf", "..."]},
    {"id": "image", "port": 8188, "running": true,  "models": ["flux1-dev", "sdxl"]},
    {"id": "voice", "port": 8284, "running": false, "models": ["piper-it-paola"]}
  ],
  "vram": {"total_mb": 8192, "used_mb": 3521, "free_mb": 4266},
  "lease": {"holder": "image", "since": "2026-11-03T20:50:00Z"}
}
```

### `/v1/route`

```json
{
  "service": "llm",
  "model": "Qwen_Qwen3.5-9B-Q4_K_M.gguf",
  "payload": { "messages": [{"role": "user", "content": "..."}], "max_tokens": 300 }
}
```

Il router:

1. verifica il servizio richiesto;
2. se la GPU è occupata da un altro servizio, **chiede lo stop** (non uccide processi sconosciuti);
3. attende che la VRAM scenda sotto una soglia;
4. avvia il servizio richiesto;
5. attende l’health;
6. inoltra la richiesta;
7. registra `active_service`, `active_model`, `last_used`;
8. dopo N secondi di inattività, scarica (idle timeout).

### Modello come chiave di routing

Naming unico, leggibile da Nexus e dagli altri client:

```
llm:Qwen_Qwen3.5-9B-Q4_K_M.gguf
image:flux1-dev
voice:piper-it-paola
```

Nexus può inviare `model: "llm:Qwen_Qwen3.5-9B-Q4_K_M.gguf"`;
il router toglie il prefisso, carica il modello giusto e proxa
`/v1/chat/completions` su `127.0.0.1:8086`.

Per image/voice, il client chiama `/v1/route` con `service` e `payload`
specifico. Se si vuole un unico punto pubblico, è il **Node Router** a fare
da sportello, non Nexus: Nexus resta il gateway LLM.

## 4. Proprietà della VRAM

Regola: **un solo lease GPU alla volta**.

| Servizio | VRAM stimata | Può convivere con LLM? |
|---|---:|---|
| LLM 9B Q4 + TurboQuant | ~6,2–7,5 GB | no |
| ComfyUI | ~3–6 GB | no |
| Voice (Piper/TTS) | CPU o <1 GB | forse, ma non garantito |
| LFM3 test | 3,4 GB | no |

Il router deve conoscere i processi che occupano VRAM. I processi non
registrati (es. LFM3) vanno trattati come **conflitto**, non killati alla cieca:
il router risponde `409 conflict` e chiede al chiamante di liberare, oppure
usa un endpoint esplicito `POST /v1/release` con target allowlist.

## 5. Nexus

Configurazione provider da aggiornare:

| Campo | Valore |
|---|---|
| provider | `beellama-tailscale-2070` |
| base_url | `http://100.98.20.76:8080` |
| model | `Qwen_Qwen3.5-9B-Q4_K_M.gguf` |
| max_ctx | `8192` |
| tags | `chitchat`, `reasoning`, `local` |
| priority | `1` |

Il 9B va precaricato con BeeLlama:

```json
POST http://100.98.20.76:8080/v1/switch_model
{
  "model": "Qwen_Qwen3.5-9B-Q4_K_M.gguf",
  "cache_k": "turbo4",
  "cache_v": "turbo3_tcq",
  "context_size": 8192,
  "slots": 1
}
```

Se il nodo è occupato da un altro servizio, Nexus riceve errore dal provider
locale, lo mette in cooldown e passa al cloud. Il chiamante non fa retry diretti.

## 6. Ordine di implementazione

1. **Mappa congelata** (questo documento): porte, servizi, proprietà.
2. **Registry read-only** nel router: `/v1/services` e `/health` con VRAM e processi.
3. **Lease LLM**: il router gestisce solo `llama-server`, ma rifiuta se ComfyUI/LFM3 occupano VRAM.
4. **Lease image**: il router chiama `gpu_monitor` per stop/start ComfyUI.
5. **Voce**: decidere se resta Piper locale (CPU) o diventa servizio del nodo.
6. **Nexus**: aggiornare provider a 9B, tag `reasoning`, `max_ctx=8192`; test attraverso Nexus.
7. **Route image/voice**: aggiungere `/v1/route` e i proxy specifici.
8. **Idle timeout e coda**: un solo slot GPU; richieste in coda o `429`.

## 7. Decisioni aperte

- Il router sostituisce lo switcher attuale o lo estende? Consiglio: **lo estende**, mantenendo `/v1/switch_model` per compatibilità.
- La voce resta **Piper locale su CPU** o diventa un servizio GPU sul nodo? Se resta CPU, non contende VRAM.
- I processi **LFM3** sono test temporanei o servizi? Se sono test, il router deve poterli fermare con un comando esplicito.
- **8443/8444**: da identificare prima di metterli in mappa come servizi.
- Nexus deve instradare anche image/voice, o resta solo LLM? Se deve fare tutto, serve un endpoint generico in Nexus; altrimenti il punto unico è il Node Router.
