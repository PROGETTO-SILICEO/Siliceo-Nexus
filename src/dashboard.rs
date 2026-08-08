use axum::response::Html;

pub fn render_dashboard() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="it">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>💎 Siliceo-Nexus — Control Center</title>
    <style>
        :root {
            --bg: #0b0f19;
            --panel: #131b2e;
            --border: #1e293b;
            --primary: #38bdf8;
            --accent: #818cf8;
            --success: #34d399;
            --warning: #fbbf24;
            --danger: #f87171;
            --text: #f8fafc;
            --muted: #94a3b8;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; }
        body { background: var(--bg); color: var(--text); padding: 20px; line-height: 1.5; }
        .header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 20px; margin-bottom: 20px; }
        .header h1 { font-size: 1.5rem; display: flex; align-items: center; gap: 10px; color: var(--primary); }
        .tabs { display: flex; gap: 10px; margin-bottom: 20px; }
        .tab-btn { background: var(--panel); color: var(--muted); border: 1px solid var(--border); padding: 8px 16px; border-radius: 8px; font-weight: 600; cursor: pointer; }
        .tab-btn.active { background: var(--primary); color: #000; border-color: var(--primary); }
        .tab-content { display: none; }
        .tab-content.active { display: block; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px; margin-bottom: 25px; }
        .card { background: var(--panel); border: 1px solid var(--border); border-radius: 12px; padding: 20px; }
        .card h2 { font-size: 1.1rem; color: var(--accent); margin-bottom: 15px; display: flex; justify-content: space-between; align-items: center; }
        table { width: 100%; border-collapse: collapse; margin-top: 10px; }
        th, td { text-align: left; padding: 10px; border-bottom: 1px solid var(--border); font-size: 0.85rem; }
        th { color: var(--muted); font-weight: 500; }
        .badge { padding: 3px 8px; border-radius: 6px; font-size: 0.75rem; font-weight: 600; text-transform: uppercase; }
        .badge-free { background: rgba(52, 211, 153, 0.15); color: var(--success); }
        .badge-paid { background: rgba(248, 113, 113, 0.15); color: var(--danger); }
        .badge-local { background: rgba(56, 189, 248, 0.15); color: var(--primary); }
        .badge-cooldown { background: rgba(251, 191, 36, 0.15); color: var(--warning); }
        .form-group { margin-bottom: 12px; }
        .form-group label { display: block; font-size: 0.85rem; color: var(--muted); margin-bottom: 5px; }
        input, select, textarea { width: 100%; padding: 8px 12px; background: var(--bg); border: 1px solid var(--border); border-radius: 6px; color: var(--text); font-size: 0.9rem; }
        button { background: var(--primary); color: #000; border: none; padding: 8px 14px; border-radius: 6px; font-weight: 600; cursor: pointer; transition: opacity 0.2s; }
        button:hover { opacity: 0.9; }
        .btn-danger { background: var(--danger); color: #fff; }
        .btn-secondary { background: var(--border); color: var(--text); }
        .btn-edit { background: var(--accent); color: #fff; }
        .node-status { display: flex; gap: 10px; align-items: center; font-size: 0.85rem; padding: 8px 12px; background: rgba(56, 189, 248, 0.1); border-radius: 8px; border: 1px solid rgba(56, 189, 248, 0.2); }
        .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--success); display: inline-block; }
        .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; margin-bottom: 20px; }
        .stat-card { background: var(--panel); border: 1px solid var(--border); padding: 15px; border-radius: 10px; text-align: center; }
        .stat-val { font-size: 1.5rem; font-weight: bold; color: var(--primary); margin-top: 5px; }
        /* Modal Result Box */
        .modal-overlay { display: none; position: fixed; top:0; left:0; width:100%; height:100%; background: rgba(0,0,0,0.7); justify-content: center; align-items: center; z-index: 100; }
        .modal-box { background: var(--panel); border: 1px solid var(--border); border-radius: 12px; padding: 25px; width: 90%; max-width: 550px; }
        .modal-box h3 { margin-bottom: 15px; color: var(--primary); display: flex; justify-content: space-between; }
    </style>
</head>
<body>
    <div class="header">
        <h1>💎 Siliceo-Nexus <span style="font-size:0.8rem; color:var(--muted); font-weight:normal;">v0.1.0 (Port :8082)</span></h1>
        <div style="display:flex; gap:12px; align-items:center;">
            <div style="display:flex; align-items:center; gap:6px; background:rgba(255,255,255,0.05); padding:4px 10px; border-radius:6px; border:1px solid var(--border);">
                <span style="font-size:0.8rem; color:var(--muted);">🔒 Admin Token:</span>
                <input type="password" id="admin-token" placeholder="NEXUS_ADMIN_TOKEN (opzionale)" style="width:160px; padding:3px 8px; font-size:0.75rem;" oninput="localStorage.setItem('nexus_admin_token', this.value)">
            </div>
            <div class="node-status">
                <span class="dot"></span>
                <span>Tailscale GPU Node: <strong>100.98.20.76 (:8080)</strong></span>
            </div>
        </div>
    </div>

    <div class="stats-grid">
        <div class="stat-card">
            <div style="font-size:0.8rem; color:var(--muted);">Provider Configurati</div>
            <div class="stat-val" id="stat-count">0</div>
        </div>
        <div class="stat-card">
            <div style="font-size:0.8rem; color:var(--muted);">Modelli in Catalogo</div>
            <div class="stat-val" id="stat-models" style="color:var(--accent);">394</div>
        </div>
        <div class="stat-card">
            <div style="font-size:0.8rem; color:var(--muted);">Costo Stimato</div>
            <div class="stat-val" style="color:var(--success);">$0.00</div>
        </div>
        <div class="stat-card">
            <div style="font-size:0.8rem; color:var(--muted);">Rotazione Chiavi</div>
            <div class="stat-val" style="color:var(--warning);">Multi-Key Active</div>
        </div>
    </div>

    <div class="tabs" id="tabs-bar">
        <button class="tab-btn active" id="tab-btn-providers" onclick="switchTab('providers')">⚡ Providers & Pool Stack</button>
        <button class="tab-btn" id="tab-btn-cat-google_aistudio" onclick="switchTab('cat-google_aistudio')">♊ Google AI Studio</button>
        <button class="tab-btn" id="tab-btn-cat-openrouter" onclick="switchTab('cat-openrouter')">🪐 OpenRouter</button>
    </div>

    <!-- TAB 1: PROVIDERS -->
    <div id="tab-providers" class="tab-content active">
        <div class="grid">
            <div class="card" style="grid-column: span 2;">
                <h2>⚡ Stack Provider Attivi <button onclick="loadProviders()">🔄 Aggiorna</button></h2>
                <table>
                    <thead>
                        <tr>
                            <th>Nome</th>
                            <th>Modello</th>
                            <th>Tier</th>
                            <th>Prio</th>
                            <th>Capability Tags</th>
                            <th>Stato Cooldown</th>
                            <th>Azioni</th>
                        </tr>
                    </thead>
                    <tbody id="providers-body">
                        <tr><td colspan="7" style="color:var(--muted); text-align:center;">Caricamento provider...</td></tr>
                    </tbody>
                </table>
            </div>

            <div class="card">
                <h2><span id="form-title">➕ Aggiungi Provider</span></h2>
                <form id="provider-form" onsubmit="saveProvider(event)">
                    <input type="hidden" id="p-id">
                    <div class="form-group">
                        <label style="color:var(--primary); font-weight:bold;">⚡ Seleziona Preset Provider (Compilazione Automatica)</label>
                        <select id="p-preset" onchange="applyPresetProvider(this.value)" style="border-color:var(--primary);">
                            <option value="">-- Scegli o inserisci manualmente --</option>
                            <option value="groq">⚡ Groq Cloud (Ultra Fast Llama / Qwen)</option>
                            <option value="google">♊ Google AI Studio (Gemini 2.5 Pro / Flash)</option>
                            <option value="deepseek">🧠 DeepSeek (V3 / R1)</option>
                            <option value="nvidia">🟢 NVIDIA NIM / Build</option>
                            <option value="alibaba">🐉 Alibaba Cloud / Qwen (DashScope)</option>
                            <option value="anthropic">🎨 Anthropic (Claude 3.5 Sonnet / Haiku)</option>
                            <option value="openai">🤖 OpenAI (GPT-4o / o3-mini)</option>
                            <option value="aws">☁️ AWS Bedrock / Mantle Proxy</option>
                            <option value="inception">🔥 Inception / Fireworks AI</option>
                            <option value="agnes">🕊️ Agnes AI (Local / Tailscale)</option>
                            <option value="mistral">🌪️ Mistral AI</option>
                            <option value="together">🤝 Together AI</option>
                            <option value="perplexity">🔍 Perplexity AI</option>
                            <option value="cerebras">⚡ Cerebras AI</option>
                            <option value="sambanova">🟧 SambaNova Systems</option>
                            <option value="openrouter">🪐 OpenRouter Network</option>
                            <option value="ollama_local">🏠 Ollama Local (Node RTX 2070 / :8080)</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Nome Provider</label>
                        <input type="text" id="p-name" placeholder="es. groq-free-pool" required>
                    </div>
                    <div class="form-group">
                        <label>Base URL / Endpoint</label>
                        <input type="text" id="p-url" placeholder="http://100.98.20.76:8080 o https://api.groq.com/openai/v1" required>
                    </div>
                    <div class="form-group">
                        <label>API Keys (Multi-Key Pool: separa con virgola)</label>
                        <textarea id="p-key" rows="2" placeholder="KEY_1, KEY_2, KEY_3 (Incolla qui per auto-riconoscimento)" oninput="detectProviderFromKey(this.value)"></textarea>
                    </div>
                    <div class="form-group">
                        <label style="display:flex; justify-content:space-between; align-items:center;">
                            <span>Modello Target</span>
                            <button type="button" class="btn-secondary" style="padding:2px 8px; font-size:0.75rem; background:rgba(56,189,248,0.15); color:var(--primary); border:1px solid var(--primary);" onclick="fetchLiveModelsFromEndpoint()">🔍 Rileva Modelli dal Vivo</button>
                        </label>
                        <div style="display:flex; gap:6px; flex-direction:column; margin-top:4px;">
                            <input type="text" id="p-model" placeholder="es. llama-3.3-70b-versatile o gemini-2.5-flash" required>
                            <select id="p-model-select" style="display:none;" onchange="if(this.value){ document.getElementById('p-model').value = this.value; }"></select>
                        </div>
                    </div>
                    <div class="form-group">
                        <label>Tier di Costo</label>
                        <select id="p-tier">
                            <option value="local">Local (RTX 2070 - $0)</option>
                            <option value="free" selected>Free (Cloud Stack - $0)</option>
                            <option value="paid">Paid (On-Demand / Backup)</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Priorità (1 = Massima)</label>
                        <input type="number" id="p-priority" value="1" min="1">
                    </div>
                    <div class="form-group">
                        <label>Capabilities / Tag (separati da virgola)</label>
                        <input type="text" id="p-tags" placeholder="coding, chitchat, tool_supported">
                    </div>
                    <button type="submit" id="btn-save">Salva Provider nel Nexus</button>
                    <button type="button" class="btn-secondary" id="btn-cancel" style="display:none;" onclick="resetForm()">Annulla</button>
                </form>
            </div>
        </div>
    </div>

    <!-- TAB 2: CATALOGO MODELLI (DINAMICO) -->
    <div id="tab-catalog" class="tab-content">
        <div class="card">
            <h2><span id="catalog-title">📚 Catalogo Modelli</span> 
                <button onclick="syncCatalog()">🔄 Sincronizza Cataloghi Ora</button>
            </h2>
            <div style="display:flex; flex-wrap:wrap; gap:15px; margin-bottom:15px; align-items:center;">
                <input type="text" id="catalog-search" style="flex:1; min-width:200px;" placeholder="🔍 Cerca modello (es. gemini, qwen, llama, deepseek)..." oninput="filterCatalog()">
                <select id="catalog-filter" onchange="filterCatalog()" style="width:180px;">
                    <option value="all">Tutti i Modelli</option>
                    <option value="free" selected>Solo 100% Free ($0.00)</option>
                </select>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>ID Modello</th>
                        <th>Sorgente</th>
                        <th>Costo Prompt / 1M</th>
                        <th>Costo Comp / 1M</th>
                        <th>Contesto</th>
                        <th>Gratuito</th>
                        <th>Azione</th>
                    </tr>
                </thead>
                <tbody id="catalog-body">
                    <tr><td colspan="7" style="color:var(--muted); text-align:center;">Caricamento catalogo...</td></tr>
                </tbody>
            </table>
        </div>
    </div>

    <!-- MODAL TEST RESULT -->
    <div id="test-modal" class="modal-overlay">
        <div class="modal-box">
            <h3><span>🧪 Risultato Test Connettività</span> <button class="btn-secondary" onclick="closeTestModal()">✕</button></h3>
            <div id="test-content">
                <p style="color:var(--muted)">Esecuzione test in corso...</p>
            </div>
        </div>
    </div>

    <script>
        let fullCatalog = [];
        let loadedProvidersList = [];
        let catalogProvidersMeta = [];
        let currentCatalogSource = 'all';
        let activeTabKey = 'providers';

        const PRESETS = {
            groq: {
                key: "groq",
                name: "groq-free-pool",
                url: "https://api.groq.com/openai/v1",
                tier: "free",
                priority: 1,
                tags: "fast, cloud_free, coding, chitchat",
                default_model: "llama-3.3-70b-versatile"
            },
            google: {
                key: "google",
                name: "gemini-free-tier",
                url: "https://generativelanguage.googleapis.com/v1beta/openai",
                tier: "free",
                priority: 1,
                tags: "chitchat, coding, fast, cloud_free, tool_supported",
                default_model: "gemini-2.5-flash"
            },
            deepseek: {
                key: "deepseek",
                name: "deepseek-official",
                url: "https://api.deepseek.com/v1",
                tier: "paid",
                priority: 1,
                tags: "coding, reasoning, deepseek_r1",
                default_model: "deepseek-chat"
            },
            nvidia: {
                key: "nvidia",
                name: "nvidia-nim-build",
                url: "https://integrate.api.nvidia.com/v1",
                tier: "free",
                priority: 2,
                tags: "gpu_accelerated, coding, llama3",
                default_model: "meta/llama-3.3-70b-instruct"
            },
            alibaba: {
                key: "alibaba",
                name: "alibaba-qwen-dashscope",
                url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
                tier: "paid",
                priority: 2,
                tags: "qwen_2_5, coding, multi_language",
                default_model: "qwen-max"
            },
            anthropic: {
                key: "anthropic",
                name: "anthropic-claude-api",
                url: "https://api.anthropic.com/v1",
                tier: "paid",
                priority: 1,
                tags: "claude_sonnet, coding, reasoning",
                default_model: "claude-3-5-sonnet-20241022"
            },
            openai: {
                key: "openai",
                name: "openai-official",
                url: "https://api.openai.com/v1",
                tier: "paid",
                priority: 2,
                tags: "gpt4o, reasoning, o3_mini",
                default_model: "gpt-4o-mini"
            },
            aws: {
                key: "aws",
                name: "aws-bedrock-mantle",
                url: "http://localhost:3001/v1",
                tier: "paid",
                priority: 1,
                tags: "aws_bedrock, proxy_mantle, enterprise",
                default_model: "anthropic.claude-3-5-sonnet-20241022-v2:0"
            },
            inception: {
                key: "inception",
                name: "inception-fireworks",
                url: "https://api.fireworks.ai/inference/v1",
                tier: "paid",
                priority: 2,
                tags: "fireworks_ai, fast_inference, open_models",
                default_model: "accounts/fireworks/models/deepseek-r1"
            },
            agnes: {
                key: "agnes",
                name: "agnes-ai-singapore",
                url: "https://apihub.agnes-ai.com/v1",
                tier: "free",
                priority: 1,
                tags: "agnes_ai, omni_modal, singapore_cloud, free_api",
                default_model: "agnes-v1"
            },
            mistral: {
                key: "mistral",
                name: "mistral-official",
                url: "https://api.mistral.ai/v1",
                tier: "free",
                priority: 2,
                tags: "mistral_large, codestral, free_tier",
                default_model: "codestral-latest"
            },
            together: {
                key: "together",
                name: "together-ai",
                url: "https://api.together.xyz/v1",
                tier: "paid",
                priority: 2,
                tags: "open_source_models, llama3_1, qwen2_5",
                default_model: "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo"
            },
            perplexity: {
                key: "perplexity",
                name: "perplexity-sonar",
                url: "https://api.perplexity.ai",
                tier: "paid",
                priority: 2,
                tags: "search_grounded, sonar_pro, web_search",
                default_model: "sonar-pro"
            },
            cerebras: {
                key: "cerebras",
                name: "cerebras-fast-ai",
                url: "https://api.cerebras.ai/v1",
                tier: "free",
                priority: 1,
                tags: "ultra_fast, wafer_scale, llama3_1",
                default_model: "llama3.1-70b"
            },
            sambanova: {
                key: "sambanova",
                name: "sambanova-systems",
                url: "https://api.sambanova.ai/v1",
                tier: "free",
                priority: 1,
                tags: "sambanova_rdu, fast_llama, deepseek_r1",
                default_model: "Meta-Llama-3.3-70B-Instruct"
            },
            openrouter: {
                key: "openrouter",
                name: "openrouter-free-pool",
                url: "https://openrouter.ai/api/v1",
                tier: "free",
                priority: 2,
                tags: "chitchat, coding, openrouter_pool",
                default_model: "qwen/qwen-2.5-coder-32b:free"
            },
            ollama_local: {
                key: "ollama_local",
                name: "ollama-local-gpu",
                url: "http://100.98.20.76:8080/v1",
                tier: "local",
                priority: 1,
                tags: "rtx_2070, local_gpu, tailscale",
                default_model: "qwen2.5-coder:32b"
            }
        };

        function applyPresetProvider(presetKey) {
            if (!presetKey || !PRESETS[presetKey]) return;
            const p = PRESETS[presetKey];
            document.getElementById('p-name').value = p.name;
            document.getElementById('p-url').value = p.url;
            document.getElementById('p-model').value = p.default_model;
            document.getElementById('p-tier').value = p.tier;
            document.getElementById('p-priority').value = p.priority;
            document.getElementById('p-tags').value = p.tags;
        }

        function detectProviderFromKey(keyVal) {
            if (!keyVal) return;
            const trimmed = keyVal.trim();
            let detectedPreset = null;

            if (trimmed.startsWith('gsk_')) detectedPreset = 'groq';
            else if (trimmed.startsWith('AIzaSy')) detectedPreset = 'google';
            else if (trimmed.startsWith('sk-ant-')) detectedPreset = 'anthropic';
            else if (trimmed.startsWith('nvapi-')) detectedPreset = 'nvidia';
            else if (trimmed.startsWith('sk-proj-')) detectedPreset = 'openai';
            else if (trimmed.startsWith('sk-or-')) detectedPreset = 'openrouter';
            else if (trimmed.startsWith('pplx-')) detectedPreset = 'perplexity';
            else if (trimmed.startsWith('csk-')) detectedPreset = 'cerebras';
            else if (trimmed.startsWith('fw_')) detectedPreset = 'inception';
            else if (trimmed.includes('agnes')) detectedPreset = 'agnes';

            if (detectedPreset) {
                document.getElementById('p-preset').value = detectedPreset;
                applyPresetProvider(detectedPreset);
            }
        }

        function getAuthHeaders() {
            const tokenInput = document.getElementById('admin-token');
            const token = (tokenInput && tokenInput.value) ? tokenInput.value : (localStorage.getItem('nexus_admin_token') || '');
            const headers = { 'Content-Type': 'application/json' };
            if (token && token.trim()) {
                headers['Authorization'] = 'Bearer ' + token.trim();
            }
            return headers;
        }

        if (localStorage.getItem('nexus_admin_token')) {
            const tokenInput = document.getElementById('admin-token');
            if (tokenInput) tokenInput.value = localStorage.getItem('nexus_admin_token');
        }

        async function fetchLiveModelsFromEndpoint() {
            const baseUrl = document.getElementById('p-url').value;
            const apiKey = document.getElementById('p-key').value;
            const presetVal = document.getElementById('p-preset').value;
            const provKey = presetVal || 'custom';

            if (!baseUrl) {
                alert("⚠️ Inserisci prima l'Endpoint o seleziona un Preset Provider.");
                return;
            }

            const modelInput = document.getElementById('p-model');
            const modelSelect = document.getElementById('p-model-select');
            modelInput.placeholder = "🔍 Download modelli in corso dall'endpoint...";

            try {
                const res = await fetch('/providers/fetch_models', {
                    method: 'POST',
                    headers: getAuthHeaders(),
                    body: JSON.stringify({ base_url: baseUrl, api_key: apiKey || null, provider_key: provKey })
                });

                const data = await res.json();
                if (res.ok && data.models && data.models.length > 0) {
                    modelSelect.innerHTML = '<option value="">-- Seleziona uno dei ' + data.models.length + ' modelli rilevati dal vivo --</option>';
                    data.models.forEach(m => {
                        modelSelect.innerHTML += `<option value="${m}">${m}</option>`;
                    });
                    modelSelect.style.display = 'block';
                    modelInput.value = data.models[0];
                    alert(`✅ Rilevati dal vivo ${data.models.length} modelli da '${data.endpoint_used}'!\n• Modelli sincronizzati nel Catalogo.`);
                    loadCatalog();
                } else {
                    alert("⚠️ Errore rilevamento modelli: " + (data.error || "Nessun modello restituito dall'endpoint."));
                    modelInput.placeholder = "qwen/qwen-2.5-coder-32b:free";
                }
            } catch(e) {
                alert("⚠️ Errore di connessione all'endpoint: " + e);
                modelInput.placeholder = "qwen/qwen-2.5-coder-32b:free";
            }
        }

        function renderDynamicTabs() {
            const container = document.getElementById('tabs-bar');
            let html = `<button class="tab-btn ${activeTabKey === 'providers' ? 'active' : ''}" id="tab-btn-providers" onclick="switchTab('providers')">⚡ Stack Provider Attivi</button>`;
            
            catalogProvidersMeta.forEach(p => {
                const tabId = `cat-${p.key}`;
                const isActive = activeTabKey === tabId;
                html += `<button class="tab-btn ${isActive ? 'active' : ''}" id="tab-btn-${tabId}" onclick="switchTab('${tabId}')">${p.label} (${p.count})</button>`;
            });

            container.innerHTML = html;
        }

        function switchTab(tabKey) {
            activeTabKey = tabKey;
            renderDynamicTabs();

            const providersTabContent = document.getElementById('tab-providers');
            const catalogTabContent = document.getElementById('tab-catalog');

            if (tabKey === 'providers') {
                providersTabContent.classList.add('active');
                catalogTabContent.classList.remove('active');
            } else if (tabKey.startsWith('cat-')) {
                const providerKey = tabKey.replace('cat-', '');
                currentCatalogSource = providerKey;
                providersTabContent.classList.remove('active');
                catalogTabContent.classList.add('active');
                
                const provMeta = catalogProvidersMeta.find(p => p.key === providerKey);
                const titleText = provMeta ? `${provMeta.label} (${provMeta.count} modelli)` : 'Catalogo Modelli';
                document.getElementById('catalog-title').innerText = titleText;
                
                filterCatalog();
            }
        }

        async function loadProviders() {
            try {
                const res = await fetch('/providers');
                const data = await res.json();
                loadedProvidersList = data.providers || [];
                const tbody = document.getElementById('providers-body');
                tbody.innerHTML = '';

                if (loadedProvidersList.length === 0) {
                    tbody.innerHTML = '<tr><td colspan="7" style="color:var(--muted); text-align:center;">Nessun provider configurato.</td></tr>';
                    return;
                }

                document.getElementById('stat-count').innerText = loadedProvidersList.length;

                loadedProvidersList.forEach(p => {
                    const badgeClass = p.tier === 'local' ? 'badge-local' : (p.tier === 'free' ? 'badge-free' : 'badge-paid');
                    const tags = (p.tags || []).map(t => `<span style="font-size:0.7rem; background:#1e293b; padding:2px 5px; border-radius:4px; margin-right:3px;">${t}</span>`).join('');
                    const isCooldown = p.cooldown_until && new Date(p.cooldown_until) > new Date();
                    const status = isCooldown 
                        ? `<span class="badge badge-cooldown">⏳ Cooldown</span>` 
                        : (p.enabled ? '🟢 Attivo' : '🔴 Disabilitato');

                    tbody.innerHTML += `
                        <tr>
                            <td><strong>${p.name}</strong></td>
                            <td><code>${p.model}</code></td>
                            <td><span class="badge ${badgeClass}">${p.tier}</span></td>
                            <td>P${p.priority}</td>
                            <td>${tags}</td>
                            <td>${status}</td>
                            <td style="white-space:nowrap;">
                                <button class="btn-secondary" style="padding:4px 8px; font-size:0.75rem;" onclick="testProvider(${p.id})">🧪 Test</button>
                                <button class="btn-edit" style="padding:4px 8px; font-size:0.75rem;" onclick="editProvider(${p.id})">✏️ Modifica</button>
                                <button class="btn-danger" style="padding:4px 8px; font-size:0.75rem;" onclick="deleteProvider(${p.id})">🗑️</button>
                            </td>
                        </tr>
                    `;
                });
            } catch(e) {
                console.error("Errore caricamento provider:", e);
            }
        }

        function editProvider(id) {
            const p = loadedProvidersList.find(x => x.id === id);
            if (!p) return;
            document.getElementById('p-id').value = p.id;
            document.getElementById('p-name').value = p.name;
            document.getElementById('p-url').value = p.base_url;
            document.getElementById('p-key').value = p.api_key || '';
            document.getElementById('p-model').value = p.model;
            document.getElementById('p-tier').value = p.tier;
            document.getElementById('p-priority').value = p.priority;
            document.getElementById('p-tags').value = (p.tags || []).join(', ');
            
            document.getElementById('form-title').innerText = `✏️ Modifica '${p.name}'`;
            document.getElementById('btn-save').innerText = 'Aggiorna Provider';
            document.getElementById('btn-cancel').style.display = 'inline-block';
        }

        function resetForm() {
            document.getElementById('provider-form').reset();
            document.getElementById('p-id').value = '';
            document.getElementById('p-preset').value = '';
            document.getElementById('p-model-select').style.display = 'none';
            document.getElementById('form-title').innerText = '➕ Aggiungi Provider';
            document.getElementById('btn-save').innerText = 'Salva Provider nel Nexus';
            document.getElementById('btn-cancel').style.display = 'none';
        }

        async function saveProvider(e) {
            e.preventDefault();
            const editId = document.getElementById('p-id').value;
            
            if (editId) {
                await fetch(`/providers/${editId}`, { method: 'DELETE', headers: getAuthHeaders() });
            }

            const tags = document.getElementById('p-tags').value.split(',').map(t => t.trim()).filter(Boolean);
            const payload = {
                name: document.getElementById('p-name').value,
                base_url: document.getElementById('p-url').value,
                api_key: document.getElementById('p-key').value || null,
                auth_type: "bearer",
                model: document.getElementById('p-model').value,
                priority: parseInt(document.getElementById('p-priority').value),
                tier: document.getElementById('p-tier').value,
                tags: tags,
                tpm_limit: 50000,
                rpm_limit: 30,
                enabled: true
            };

            const res = await fetch('/providers', {
                method: 'POST',
                headers: getAuthHeaders(),
                body: JSON.stringify(payload)
            });

            if (!res.ok) {
                const errData = await res.text();
                alert("⚠️ Errore salvataggio provider: " + errData);
                return;
            }

            resetForm();
            loadProviders();
        }

        async function deleteProvider(id) {
            if (confirm("Vuoi rimuovere questo provider da Siliceo-Nexus?")) {
                const res = await fetch(`/providers/${id}`, { method: 'DELETE', headers: getAuthHeaders() });
                if (!res.ok) {
                    const errData = await res.text();
                    alert("⚠️ Errore eliminazione provider: " + errData);
                    return;
                }
                loadProviders();
            }
        }

        async function testProvider(id) {
            document.getElementById('test-modal').style.display = 'flex';
            document.getElementById('test-content').innerHTML = `
                <p style="color:var(--muted); text-align:center;">🧪 Invio prompt di test in corso...<br><span style="font-size:0.8rem">Misurazione latenza ed esito...</span></p>
            `;
            try {
                const res = await fetch(`/providers/${id}/test`, { method: 'POST' });
                const data = await res.json();
                if (data.success) {
                    document.getElementById('test-content').innerHTML = `
                        <div style="background:rgba(52,211,153,0.1); border:1px solid var(--success); padding:15px; border-radius:8px;">
                            <p style="color:var(--success); font-weight:bold; margin-bottom:5px;">✅ Test Riuscito! (${data.latency_ms} ms)</p>
                            <p style="font-size:0.85rem;"><strong>Provider:</strong> ${data.provider_name}</p>
                            <p style="font-size:0.85rem;"><strong>Modello:</strong> <code>${data.model_used}</code></p>
                            <hr style="border-color:var(--border); margin:10px 0;">
                            <p style="font-size:0.85rem; color:var(--text);"><strong>Risposta:</strong> "${data.content}"</p>
                        </div>
                    `;
                } else {
                    document.getElementById('test-content').innerHTML = `
                        <div style="background:rgba(248,113,113,0.1); border:1px solid var(--danger); padding:15px; border-radius:8px;">
                            <p style="color:var(--danger); font-weight:bold; margin-bottom:5px;">❌ Test Fallito (${data.latency_ms} ms)</p>
                            <p style="font-size:0.85rem;"><strong>Provider:</strong> ${data.provider_name}</p>
                            <p style="font-size:0.85rem; color:var(--danger);"><strong>Errore:</strong> ${data.error}</p>
                        </div>
                    `;
                }
            } catch(e) {
                document.getElementById('test-content').innerHTML = `<p style="color:var(--danger)">Errore di rete durante il test: ${e}</p>`;
            }
        }

        function closeTestModal() {
            document.getElementById('test-modal').style.display = 'none';
        }

        async function loadCatalog() {
            try {
                const res = await fetch('/catalog');
                const data = await res.json();
                fullCatalog = data.catalog || [];
                catalogProvidersMeta = data.catalog_providers || [];

                let statsBreakdown = [];
                catalogProvidersMeta.forEach(p => {
                    statsBreakdown.push(`${p.label.split(' ')[0]} ${p.count}`);
                });

                document.getElementById('stat-models').innerHTML = `${fullCatalog.length} <span style="font-size:0.75rem; font-weight:normal; color:var(--muted);">(${statsBreakdown.join(' | ')})</span>`;

                renderDynamicTabs();
                if (activeTabKey !== 'providers') {
                    filterCatalog();
                }
            } catch(e) {
                console.error("Errore caricamento catalogo:", e);
            }
        }

        function filterCatalog() {
            const query = document.getElementById('catalog-search').value.toLowerCase();
            const filterType = document.getElementById('catalog-filter').value;
            const tbody = document.getElementById('catalog-body');
            tbody.innerHTML = '';

            const filtered = fullCatalog.filter(m => {
                const matchesQuery = m.model_id.toLowerCase().includes(query);
                const matchesFilter = filterType === 'all' || (filterType === 'free' && m.is_free);
                const matchesSource = currentCatalogSource === 'all' || m.provider_name === currentCatalogSource;
                return matchesQuery && matchesFilter && matchesSource;
            });

            if (filtered.length === 0) {
                tbody.innerHTML = '<tr><td colspan="7" style="color:var(--muted); text-align:center;">Nessun modello trovato per i filtri selezionati.</td></tr>';
                return;
            }

            filtered.slice(0, 200).forEach(m => {
                const isFreeBadge = m.is_free ? '<span class="badge badge-free">FREE</span>' : '<span class="badge badge-paid">PAID</span>';
                const sourceBadge = `<span class="badge" style="background:rgba(56,189,248,0.2); color:#38bdf8;">${m.provider_name}</span>`;
                const ctx = (m.context_length / 1024).toFixed(0) + 'k';
                tbody.innerHTML += `
                    <tr>
                        <td><code>${m.model_id}</code></td>
                        <td>${sourceBadge}</td>
                        <td>$${m.prompt_cost_per_1m.toFixed(4)}</td>
                        <td>$${m.completion_cost_per_1m.toFixed(4)}</td>
                        <td>${ctx} tokens</td>
                        <td>${isFreeBadge}</td>
                        <td>
                            <button class="btn-secondary" style="padding:3px 8px; font-size:0.75rem;" onclick="useModelInForm('${m.model_id}', '${m.provider_name}')">➕ Usa</button>
                        </td>
                    </tr>
                `;
            });
        }

        function useModelInForm(modelId, providerName) {
            switchTab('providers');
            document.getElementById('p-model').value = modelId;
            if (providerName === 'google_aistudio' || providerName === 'google') {
                applyPresetProvider('google');
                document.getElementById('p-model').value = modelId;
            } else if (PRESETS[providerName]) {
                applyPresetProvider(providerName);
                document.getElementById('p-model').value = modelId;
            } else if (modelId.includes(':free')) {
                document.getElementById('p-tier').value = 'free';
            }
        }

        async function syncCatalog() {
            alert("🔄 Sincronizzazione cataloghi (OpenRouter + Google AI Studio) avviata...");
            const res = await fetch('/catalog/sync', { method: 'POST' });
            const data = await res.json();
            alert(`✅ Sincronizzazione completata!\n• OpenRouter: ${data.openrouter_count} modelli\n• Google AI Studio: ${data.google_count} modelli`);
            loadCatalog();
        }

        loadProviders();
        loadCatalog();
    </script>
</body>
</html>"#)
}
