# 💎 Siliceo-Nexus

![Siliceo-Nexus Banner](assets/banner.png)

> **Universal Multimodal Inference Gateway, Task-Aware Router & Preset Auto-Discovery Engine**  
> High-performance, zero-hardcode LLM gateway written in Rust (`:8082`). Automatically routes requests to local GPU nodes or cloud providers based on task intent, model capabilities, cost constraints, and multi-key round-robin rotation.

---

## 🌟 Overview

**Siliceo-Nexus** is a self-hosted, lightweight OpenAI-compatible Inference Gateway designed to decouple AI applications, autonomous agents, and IDE tools from single LLM vendors.

It abstracts provider complexities by offering a single endpoint (`http://localhost:8082/v1/chat/completions`) that dynamically selects the best inference backend—whether it is a local GPU node running Ollama/vLLM/llama.cpp or a pool of free/paid cloud API providers.

---

## ✨ Key Features

- **⚡ Instant Intent Classification (< 1ms)**: Automatically categorizes incoming prompts into `chitchat`, `coding`, `reasoning`, or `tool_call` without adding latency.
- **📈 Live Hardware & Gateway Telemetry (`GET /stats`)**: Real-time GPU load, system RAM utilization, and latency sparkline graphs polled live on the Web Dashboard.
- **🔄 1-Click Model Hot-Swapping (`POST /providers/:id/set_model`)**: Dynamically switch the active target model for any local or cloud provider instantly with 1-click on the dashboard, zero service restarts required.
- **🐝 `beellama-switcher` Zero-GC Rust Micro-Daemon**: Standalone 2MB micro-service for dedicated GPU inference nodes (RTX 2070 / TurboQuant / GGUF). Scans local GGUF directories, exposes `/v1/models`, and manages `llama-server` process hot-swapping on demand with <5MB RAM footprint.
- **🎨 Native Anthropic Messages API (`/v1/messages`)**: Full compatibility with Claude Code CLI, Claude Desktop, and Anthropic SDKs alongside standard OpenAI API routes (`/v1/chat/completions`).
- **🔒 Enterprise Security & Privacy**:
  - **Key Masking**: API keys are masked over the network (`gsk_...9a2f`). Raw plaintext keys are never returned in JSON.
  - **Cross-Platform Storage Security**: Enforces `0o600` permissions on Unix/Linux/macOS and isolated user ACLs on Windows.
  - **SSRF Protection**: Blocks requests to cloud metadata services (`169.254.169.254`, GCP metadata).
  - **Zero-Leak Log Redaction**: Automatically scrubs API key patterns from system logs.
  - **Optional Admin Auth**: Protects mutation endpoints with `NEXUS_ADMIN_TOKEN`.
- **⚡ 17 Preset Providers & Key Auto-Detection**: Instant pre-filling for Groq, Google AI Studio, DeepSeek, NVIDIA NIM, Alibaba Qwen, Anthropic, OpenAI, AWS Bedrock, Inception/Fireworks, Agnes AI (Singapore), Mistral, Together, Perplexity, Cerebras, SambaNova, OpenRouter, and Ollama Local. Auto-detects provider by pasting API key.
- **🔍 Live Model Autodiscovery (`POST /providers/fetch_models`)**: Fetch live models directly from provider endpoints and dynamically sync them to your catalog.
- **📚 Modular Dynamic Catalog Tabs**: Dynamic rendering per provider (450+ models supported).
- **🔑 Multi-Key Round-Robin Rotation**: Supports comma-separated API key pools per provider. Automatically rotates keys and handles `429 Too Many Requests` without interrupting sessions.
- **🏷️ Tag & Capability-Based Routing**: Zero hardcoded model names. Requests are routed dynamically based on capabilities (`coding`, `fast`, `local`, `tool_supported`).
- **💰 Free-Tier Aggregation & Cost Awareness**: Stacks multiple free-tier cloud API keys alongside local GPU resources to maximize throughput at zero cost ($0 USD).
- **🎛️ Integrated Web Dashboard**: Single-Page Application served at `http://localhost:8082/` for live multi-key management, priority tuning, hardware telemetry, and hot-reloading without restarts.
- **💾 SQLite WAL Engine**: Data persistence (`data/nexus.db`) configured with Write-Ahead Logging (`PRAGMA journal_mode=WAL`), transactions, and busy timeouts for high-throughput concurrent access.

---

## 🏛️ System Architecture

```
┌────────────────────────────────┐
│ CLIENT APPLICATIONS & AGENTS   │
│ - Autonomous AI Agents         │
│ - IDE Extensions / Claude Code │  HTTP OpenAI Standard
│ - Web / Chat Interfaces        │───────────────────────┐
└────────────────────────────────┘                       │
                                                         ▼
                                             💎 Siliceo-Nexus Gateway (:8082)
                                             ├─► Intent Classifier (<1ms)
                                             ├─► Multi-Key Round-Robin Selector
                                             ├─► Live 450+ Catalog Sync (24h)
                                             ├─► Preset Registry & Key Auto-Detect
                                             ├─► SSRF & Key Masking Shield
                                             └─► Dynamic Tag Selector
                                                         │
         ┌───────────────────────────────────────────────┼───────────────────────────────────────────────┐
         │                                               │                                               │
         ▼                                               ▼                                               ▼
┌─────────────────────────────────┐             ┌─────────────────────────────────┐             ┌─────────────────────────────────┐
│ TIER 0: LOCAL GPU NODE          │             │ TIER 1: FREE-TIER CLOUD POOL    │             │ TIER 2: HEAVY REASONING (PAID)  │
│ - Ollama / vLLM / llama.cpp     │             │ - Gemini Free Multi-Key Pool    │             │ - Claude 3.5 / DeepSeek R1      │
│ - Low latencies, total privacy  │             │ - Groq / Agnes AI / Cerebras    │             │ - Activated on-demand or for    │
│ - GPU VRAM Semaphore protection │             │ - Auto-cooldown on 429 errors   │             │   explicit CRITICAL tasks       │
└─────────────────────────────────┘             └─────────────────────────────────┘             └─────────────────────────────────┘
```

---

## 🎛️ Web Control Dashboard (`http://localhost:8082/`)

The integrated Web SPA provides complete governance over your inference infrastructure:

1. **Preset Provider Registry & Auto-Detection**: Select from 17 pre-configured providers or paste an API key to auto-detect.
2. **Live Autodiscovery**: Query provider `/models` endpoints to discover and sync models live.
3. **Multi-Key Management**: Pair multiple keys for the same provider (e.g. `KEY_1, KEY_2, KEY_3`).
4. **Dynamic Catalog Tabs**: Switch between provider catalogs (Google AI Studio, OpenRouter, Groq, etc.).

## 🔌 Connecting Claude Code, Cursor & Aider

Siliceo-Nexus works seamlessly with **Claude Code**, **Cursor**, **Aider**, **Windsurf**, and **Claude CLI**.

### 1. Claude Code CLI Integration
To direct **Claude Code** through Siliceo-Nexus (`:8082`):

```bash
# Set environment variables in your terminal
export ANTHROPIC_BASE_URL="http://localhost:8082/v1"
export ANTHROPIC_API_KEY="nexus-local"

# Launch Claude Code — all prompts will route through Siliceo-Nexus
claude
```

**Pro Tip (One-Liner Alias)**: Add to `~/.bashrc` or `~/.zshrc`:
```bash
alias claude-nexus='ANTHROPIC_BASE_URL="http://localhost:8082/v1" ANTHROPIC_API_KEY="nexus-local" claude'
```

### 2. Cursor / Windsurf IDE Setup
In Cursor / Windsurf settings:
- **OpenAI API Key**: `nexus-local`
- **Override OpenAI Base URL**: `http://localhost:8082/v1`

---

## 📡 API Reference

Siliceo-Nexus implements standard OpenAI API specifications:

- `POST /v1/chat/completions` — Text generation, Chat, and Function Calling
- `GET /catalog` — Live catalog of 450+ models with costs and context lengths
- `POST /catalog/sync` — Trigger immediate catalog synchronization
- `GET /providers` — List configured providers (API keys masked)
- `POST /providers` — Register or update a provider dynamically
- `POST /providers/fetch_models` — Fetch live models from endpoint & sync to catalog
- `DELETE /providers/:id` — Remove a provider entry
- `POST /providers/:id/test` — Run connectivity test prompt
- `GET /health` — Service health check

---

## 🛠️ Quick Start

### Prerequisites
- Rust (Cargo) 1.75+

### Installation & Run

```bash
# 1. Clone repository
git clone https://github.com/PROGETTO-SILICEO/Siliceo-Nexus.git
cd Siliceo-Nexus

# 2. Build release binary
cargo build --release

# 3. Launch gateway
./target/release/siliceo-nexus
```

Access the Web Control Panel at `http://localhost:8082/`.
