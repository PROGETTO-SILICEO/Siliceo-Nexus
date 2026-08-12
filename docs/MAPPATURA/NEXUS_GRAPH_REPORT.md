# Graph Report - .  (2026-08-10)

## Corpus Check
- 16 files · ~96,897 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 179 nodes · 432 edges · 19 communities (13 shown, 6 thin omitted)
- Extraction: 96% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Adapter & Provider Dispatch
- beellama-switcher
- Security & SSRF
- Management API
- Catalog Sync
- Concepts Gateway
- Database SQLite
- Router & Intent
- AppState & Core
- Dashboard API & Health
- Infrastruttura Rete
- Anthropic Messages
- Web Dashboard UI
- Install Shell
- Free-Tier
- Presets
- SSRF Concept
- Systemd

## God Nodes (most connected - your core abstractions)
1. `AppState` - 23 edges
2. `verify_admin_auth()` - 13 edges
3. `create_provider()` - 13 edges
4. `handle_fetch_models()` - 13 edges
5. `AppState` - 12 edges
6. `handle_chat_completions()` - 12 edges
7. `handle_set_provider_model()` - 12 edges
8. `Provider` - 12 edges
9. `LLMRequest` - 12 edges
10. `handle_v1_models()` - 11 edges

## Surprising Connections (you probably didn't know these)
- `insert_provider_db()` --references--> `ProviderInput`  [EXTRACTED]
  src/db.rs → src/types.rs
- `load_all_providers()` --references--> `Provider`  [EXTRACTED]
  src/db.rs → src/types.rs
- `AppState` --references--> `Provider`  [EXTRACTED]
  src/main.rs → src/types.rs
- `handle_chat_completions()` --references--> `LLMRequest`  [EXTRACTED]
  src/main.rs → src/types.rs
- `handle_chat_completions()` --references--> `LLMResponse`  [EXTRACTED]
  src/main.rs → src/types.rs

## Import Cycles
- None detected.

## Communities (19 total, 6 thin omitted)

### Community 0 - "Adapter & Provider Dispatch"
Cohesion: 0.17
Nodes (24): Error, dispatch_request(), pick_api_key(), Client, Option, Result, String, try_anthropic() (+16 more)

### Community 1 - "beellama-switcher"
Cohesion: 0.17
Nodes (22): AppState, handle_health(), handle_list_models(), handle_proxy_chat(), handle_switch_model(), main(), ModelItem, Arc (+14 more)

### Community 2 - "Security & SSRF"
Cohesion: 0.15
Nodes (15): IpAddr, configured_trusted_endpoint_hosts(), ensure_network_auth_is_configured(), FetchModelsPayload, handle_fetch_models(), infer_provider_key(), inference_rate_limit(), is_loopback_bind() (+7 more)

### Community 3 - "Management API"
Cohesion: 0.41
Nodes (19): Path, delete_provider(), enforce_inference_rate_limit(), handle_chat_completions(), handle_get_catalog(), handle_set_provider_model(), handle_sync_catalog(), handle_test_provider() (+11 more)

### Community 4 - "Catalog Sync"
Cohesion: 0.28
Nodes (15): GoogleModelItem, GoogleModelsResponse, OpenRouterModelItem, OpenRouterModelsResponse, OpenRouterPricing, Client, Option, Result (+7 more)

### Community 5 - "Concepts Gateway"
Cohesion: 0.14
Nodes (14): Native Anthropic Messages API, Cascade Failover per Priorita, Catalog Sync, Cooldown su Errori 429, Intent Classification, Key Masking, Live Model Autodiscovery, Catalogo Modelli (+6 more)

### Community 6 - "Database SQLite"
Cohesion: 0.32
Nodes (11): init_db(), insert_provider_db(), load_all_providers(), mask_api_key(), Result, SqlitePool, String, Vec (+3 more)

### Community 7 - "Router & Intent"
Cohesion: 0.38
Nodes (9): classify_intent(), IntentTag, Arc, Result, RwLock, String, Vec, select_eligible_providers() (+1 more)

### Community 8 - "AppState & Core"
Cohesion: 0.25
Nodes (8): Instant, AppState, Arc, Client, Mutex, RwLock, SqlitePool, VecDeque

### Community 9 - "Dashboard API & Health"
Cohesion: 0.33
Nodes (7): create_provider(), handle_dashboard(), handle_health(), handle_stats(), redact_secrets(), IntoResponse, Json

### Community 10 - "Infrastruttura Rete"
Cohesion: 0.40
Nodes (5): beellama-switcher, Nodo Inferenza GPU, Network Profiles, Rate Limit Globale, Tailscale Mesh

### Community 11 - "Anthropic Messages"
Cohesion: 0.83
Nodes (4): AnthropicMessage, AnthropicMessageRequest, handle_anthropic_messages(), Value

## Knowledge Gaps
- **1 isolated node(s):** `install.sh script`
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `AppState & Core` to `Adapter & Provider Dispatch`, `Security & SSRF`, `Management API`, `Dashboard API & Health`, `Anthropic Messages`?**
  _High betweenness centrality (0.079) - this node is a cross-community bridge._
- **Why does `Provider` connect `Adapter & Provider Dispatch` to `AppState & Core`, `Database SQLite`, `Router & Intent`?**
  _High betweenness centrality (0.078) - this node is a cross-community bridge._
- **Why does `LLMRequest` connect `Adapter & Provider Dispatch` to `Management API`, `Router & Intent`?**
  _High betweenness centrality (0.028) - this node is a cross-community bridge._
- **What connects `install.sh script` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Concepts Gateway` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._