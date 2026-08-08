# 💎 Siliceo-Nexus

![Siliceo-Nexus Banner](assets/banner.png)

> **Universal Multimodal Inference Gateway & Task-Aware Routing Engine**  
> High-performance, zero-hardcode LLM gateway written in Rust (`:8082`). Automatically routes requests to local GPU nodes or cloud providers based on task intent, model capabilities, cost constraints, and multi-key round-robin rotation.

---

## 🌟 Overview

**Siliceo-Nexus** is a self-hosted, lightweight OpenAI-compatible Inference Gateway designed to decouple AI applications, autonomous agents, and IDE tools from single LLM vendors.

It abstracts provider complexities by offering a single endpoint (`http://localhost:8082/v1/chat/completions`) that dynamically selects the best inference backend—whether it is a local GPU node running Ollama/vLLM/llama.cpp or a pool of free/paid cloud API providers.

---

## ✨ Key Features

- **⚡ Instant Intent Classification (< 1ms)**: Automatically categorizes incoming prompts into `chitchat`, `coding`, `reasoning`, or `tool_call` without adding latency.
- **🔑 Multi-Key Round-Robin Rotation**: Supports comma-separated API key pools per provider. Automatically rotates keys and handles `429 Too Many Requests` without interrupting sessions.
- **📦 Live Model Catalog Sync (390+ Models)**: Automatically downloads and refreshes model metadata every 24 hours (costs per 1M tokens, context size, free status).
- **🏷️ Tag & Capability-Based Routing**: Zero hardcoded model names. Requests are routed dynamically based on capabilities (`coding`, `fast`, `local`, `tool_supported`).
- **🛡️ Rate-Limit & Token-Bucket Protection**: Tracks Tokens-Per-Minute (TPM) in addition to Requests-Per-Minute (RPM). Automatically places failing keys into temporary cooldowns.
- **💰 Free-Tier Aggregation & Cost Awareness**: Stacks multiple free-tier cloud API keys alongside local GPU resources to maximize throughput at zero cost ($0 USD).
- **🎛️ Integrated Web Dashboard**: Single-Page Application served at `http://localhost:8082/` for live multi-key management (Free vs Paid), priority tuning, and hot-reloading without restarts.
- **💾 SQLite WAL Engine**: Data persistence (`data/nexus.db`) configured with Write-Ahead Logging (`PRAGMA journal_mode=WAL`) and busy timeouts for high-throughput concurrent access.

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
                                             ├─► Live 390+ Catalog Sync (24h)
                                             └─► Dynamic Tag Selector
                                                         │
         ┌───────────────────────────────────────────────┼───────────────────────────────────────────────┐
         │                                               │                                               │
         ▼                                               ▼                                               ▼
┌─────────────────────────────────┐             ┌─────────────────────────────────┐             ┌─────────────────────────────────┐
│ TIER 0: LOCAL GPU NODE          │             │ TIER 1: FREE-TIER CLOUD POOL    │             │ TIER 2: HEAVY REASONING (PAID)  │
│ - Ollama / vLLM / llama.cpp     │             │ - Gemini Free Multi-Key Pool    │             │ - Claude 3.5 / DeepSeek R1      │
│ - Low latencies, total privacy  │             │ - Groq / Cerebras / SambaNova   │             │ - Activated on-demand or for    │
│ - GPU VRAM Semaphore protection │             │ - Auto-cooldown on 429 errors   │             │   explicit CRITICAL tasks       │
└─────────────────────────────────┘             └─────────────────────────────────┘             └─────────────────────────────────┘
```

---

## 🎛️ Web Control Dashboard (`http://localhost:8082/`)

The integrated Web SPA provides complete governance over your inference infrastructure:

1. **Multi-Key Management**: Pair multiple keys for the same provider (e.g. `KEY_1, KEY_2, KEY_3`).
2. **Hot Reloading**: Add, update, or disable providers on the fly without stopping active client sessions.
3. **Live Metrics & Catalog**: Monitor token consumption, active cooldowns, and explore the 390+ model catalog.

---

## 📡 API Reference

Siliceo-Nexus implements standard OpenAI API specifications:

- `POST /v1/chat/completions` — Text generation, Chat, and Function Calling
- `GET /catalog` — Live catalog of 390+ models with costs and context lengths
- `POST /catalog/sync` — Trigger immediate catalog synchronization
- `GET /providers` — List configured providers and live health statuses
- `POST /providers` — Register or update a provider dynamically
- `DELETE /providers/:id` — Remove a provider entry
- `GET /health` — Service health check

---

## 🛠️ Quick Start

### Prerequisites
- Rust (Cargo) 1.75+

### Installation & Run

```bash
# 1. Clone repository
git clone https://github.com/your-org/siliceo-nexus.git
cd siliceo-nexus

# 2. Build release binary
cargo build --release

# 3. Launch gateway
./target/release/siliceo-nexus
```

Access the Web Control Panel at `http://localhost:8082/`.
