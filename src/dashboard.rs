use axum::response::Html;

pub fn render_dashboard() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html>
<html lang="it">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>💎 Siliceo-Nexus — LLM Gateway Control Dashboard</title>
    <style>
        :root {
            --bg: #070a12;
            --panel: #0d1322;
            --card-bg: #111827;
            --card-border: #1e293b;
            --primary: #38bdf8;
            --primary-glow: rgba(56, 189, 248, 0.25);
            --accent: #818cf8;
            --success: #10b981;
            --warning: #fbbf24;
            --danger: #ef4444;
            --text: #f8fafc;
            --muted: #94a3b8;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; }
        body { background: var(--bg); color: var(--text); padding: 24px; line-height: 1.5; min-height: 100vh; }
        
        /* Top Navigation Header */
        .top-nav { display: flex; justify-content: space-between; align-items: center; background: var(--panel); border: 1px solid var(--card-border); padding: 14px 24px; border-radius: 12px; margin-bottom: 24px; box-shadow: 0 4px 20px rgba(0,0,0,0.4); }
        .logo-title { display: flex; align-items: center; gap: 12px; font-size: 1.25rem; font-weight: 700; color: var(--text); letter-spacing: -0.5px; }
        .logo-title span.tag { font-size: 0.75rem; background: rgba(56, 189, 248, 0.15); color: var(--primary); padding: 3px 8px; border-radius: 6px; border: 1px solid rgba(56, 189, 248, 0.3); }
        
        .search-box { display: flex; align-items: center; background: var(--bg); border: 1px solid var(--card-border); border-radius: 8px; padding: 6px 14px; width: 320px; gap: 8px; }
        .search-box input { background: transparent; border: none; outline: none; color: var(--text); font-size: 0.85rem; width: 100%; }

        .top-right { display: flex; align-items: center; gap: 14px; }
        .admin-box { display: flex; align-items: center; gap: 8px; background: rgba(255,255,255,0.03); border: 1px solid var(--card-border); padding: 5px 12px; border-radius: 8px; font-size: 0.8rem; }
        .admin-box input { background: transparent; border: none; outline: none; color: var(--primary); font-size: 0.8rem; width: 140px; }
        
        .node-status { display: flex; align-items: center; gap: 8px; font-size: 0.8rem; padding: 6px 12px; background: rgba(16, 185, 129, 0.1); border: 1px solid rgba(16, 185, 129, 0.25); border-radius: 8px; color: var(--success); }
        .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--success); box-shadow: 0 0 8px var(--success); }

        /* Stats Grid */
        .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 18px; margin-bottom: 20px; }
        .stat-card { background: var(--panel); border: 1px solid var(--card-border); border-radius: 12px; padding: 18px 20px; position: relative; overflow: hidden; }
        .stat-card::before { content: ''; position: absolute; top:0; left:0; width: 100%; height: 2px; background: linear-gradient(90deg, var(--primary), transparent); }
        .stat-label { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.5px; color: var(--muted); font-weight: 600; }
        .stat-val { font-size: 1.8rem; font-weight: 800; color: var(--text); margin-top: 6px; display: flex; align-items: baseline; gap: 8px; }
        .stat-val span.sub { font-size: 0.8rem; font-weight: 500; color: var(--success); }

        /* Real-Time Telemetry Panel */
        .telemetry-panel { background: var(--panel); border: 1px solid var(--card-border); border-radius: 14px; padding: 20px; margin-bottom: 24px; position: relative; box-shadow: 0 4px 20px rgba(0,0,0,0.3); }
        .telemetry-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; border-bottom: 1px solid rgba(255,255,255,0.05); padding-bottom: 10px; }
        .telemetry-title { display: flex; align-items: center; gap: 10px; font-size: 0.95rem; font-weight: 700; color: var(--primary); letter-spacing: 0.5px; }
        .live-pulse { width: 8px; height: 8px; background: var(--success); border-radius: 50%; display: inline-block; box-shadow: 0 0 10px var(--success); animation: pulse 1.5s infinite; }
        @keyframes pulse { 0% { opacity: 0.4; } 50% { opacity: 1; } 100% { opacity: 0.4; } }
        
        .telemetry-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
        .telemetry-box { background: var(--bg); border: 1px solid var(--card-border); border-radius: 10px; padding: 14px 16px; }
        .telemetry-label { font-size: 0.72rem; font-weight: 700; color: var(--muted); letter-spacing: 0.5px; text-transform: uppercase; margin-bottom: 8px; display: flex; justify-content: space-between; }
        
        .gauge-bar { height: 10px; background: rgba(255,255,255,0.05); border-radius: 6px; overflow: hidden; border: 1px solid var(--card-border); }
        .gauge-fill { height: 100%; width: 45%; background: linear-gradient(90deg, var(--primary), var(--accent)); transition: width 0.5s ease; border-radius: 6px; }
        
        .sparkline-box { display: flex; align-items: flex-end; gap: 4px; height: 32px; padding-top: 4px; }
        .spark-bar { flex: 1; background: rgba(56, 189, 248, 0.3); border-radius: 3px; transition: height 0.3s ease; }
        .spark-bar.high { background: var(--accent); }
        .spark-bar.active { background: var(--primary); box-shadow: 0 0 6px var(--primary); }

        /* Navigation Tabs */
        .nav-tabs { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--card-border); padding-bottom: 12px; margin-bottom: 24px; }
        .tabs-group { display: flex; gap: 8px; }
        .tab-btn { background: transparent; color: var(--muted); border: 1px solid transparent; padding: 8px 16px; border-radius: 8px; font-weight: 600; font-size: 0.85rem; cursor: pointer; transition: all 0.2s; }
        .tab-btn:hover { color: var(--text); background: rgba(255,255,255,0.03); }
        .tab-btn.active { background: var(--panel); color: var(--primary); border-color: var(--card-border); box-shadow: 0 2px 10px rgba(0,0,0,0.3); }

        /* Main Cards Grid */
        .cards-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(310px, 1fr)); gap: 20px; }
        .provider-card { background: var(--panel); border: 1px solid var(--card-border); border-radius: 14px; padding: 20px; transition: transform 0.2s, border-color 0.2s, box-shadow 0.2s; position: relative; }
        .provider-card:hover { transform: translateY(-3px); border-color: rgba(56, 189, 248, 0.4); box-shadow: 0 8px 25px rgba(0,0,0,0.5); }
        
        .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; }
        .provider-brand { display: flex; align-items: center; gap: 12px; }
        .provider-icon { width: 42px; height: 42px; border-radius: 10px; display: flex; align-items: center; justify-content: center; font-size: 1.2rem; font-weight: bold; background: rgba(255,255,255,0.05); border: 1px solid var(--card-border); }
        .provider-name { font-weight: 700; font-size: 1.05rem; color: var(--text); }
        .provider-sub { font-size: 0.75rem; color: var(--muted); font-family: monospace; margin-top: 2px; }
        
        .status-pill { padding: 3px 9px; border-radius: 20px; font-size: 0.7rem; font-weight: 700; text-transform: uppercase; display: flex; align-items: center; gap: 5px; }
        .status-pill.active { background: rgba(16, 185, 129, 0.15); color: var(--success); border: 1px solid rgba(16, 185, 129, 0.3); }
        .status-pill.cooldown { background: rgba(251, 191, 36, 0.15); color: var(--warning); border: 1px solid rgba(251, 191, 36, 0.3); }
        
        .card-body { margin-bottom: 16px; font-size: 0.82rem; color: var(--muted); }
        .key-badge { display: inline-flex; align-items: center; gap: 6px; background: var(--bg); border: 1px solid var(--card-border); padding: 4px 10px; border-radius: 6px; font-family: monospace; font-size: 0.78rem; color: var(--primary); margin-top: 8px; width: 100%; justify-content: space-between; }

        .card-actions { display: flex; gap: 8px; margin-top: 14px; pt-3; border-top: 1px solid rgba(255,255,255,0.05); }
        .card-actions button { flex: 1; padding: 7px; font-size: 0.75rem; border-radius: 6px; font-weight: 600; cursor: pointer; border: 1px solid var(--card-border); background: var(--bg); color: var(--text); transition: background 0.2s; }
        .card-actions button:hover { background: rgba(255,255,255,0.08); }
        .card-actions button.btn-test { background: rgba(56, 189, 248, 0.1); color: var(--primary); border-color: rgba(56, 189, 248, 0.3); }
        .card-actions button.btn-danger { color: var(--danger); }
        .card-actions button.btn-danger:hover { background: rgba(239, 68, 68, 0.15); }

        /* Buttons */
        .btn-add { background: linear-gradient(135deg, var(--primary), var(--accent)); color: #000; border: none; padding: 9px 18px; border-radius: 8px; font-weight: 700; font-size: 0.85rem; cursor: pointer; box-shadow: 0 4px 15px var(--primary-glow); }
        .btn-add:hover { opacity: 0.95; transform: scale(1.02); }

        /* Modal Overlay */
        .modal-overlay { display: none; position: fixed; top:0; left:0; width:100%; height:100%; background: rgba(0,0,0,0.75); backdrop-filter: blur(6px); justify-content: center; align-items: center; z-index: 200; }
        .modal-box { background: var(--panel); border: 1px solid var(--card-border); border-radius: 16px; padding: 28px; width: 92%; max-width: 580px; box-shadow: 0 10px 40px rgba(0,0,0,0.6); position: relative; }
        .modal-box h3 { font-size: 1.2rem; color: var(--primary); margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center; }

        .form-group { margin-bottom: 14px; }
        .form-group label { display: block; font-size: 0.8rem; color: var(--muted); margin-bottom: 6px; font-weight: 600; }
        input, select { width: 100%; padding: 9px 14px; background: var(--bg); border: 1px solid var(--card-border); border-radius: 8px; color: var(--text); font-size: 0.88rem; outline: none; }
        input:focus, select:focus { border-color: var(--primary); }

        /* Catalog Table View */
        .table-container { background: var(--panel); border: 1px solid var(--card-border); border-radius: 14px; padding: 20px; }
        table { width: 100%; border-collapse: collapse; }
        th, td { text-align: left; padding: 12px 14px; border-bottom: 1px solid var(--card-border); font-size: 0.85rem; }
        th { color: var(--muted); font-size: 0.75rem; text-transform: uppercase; font-weight: 600; }
        .badge { padding: 3px 8px; border-radius: 6px; font-size: 0.72rem; font-weight: 700; text-transform: uppercase; }
        .badge-free { background: rgba(16, 185, 129, 0.15); color: var(--success); }
        .badge-paid { background: rgba(239, 68, 68, 0.15); color: var(--danger); }
    </style>
</head>
<body>

    <!-- Top Navigation Header -->
    <div class="top-nav">
        <div class="logo-title">
            <span>💎 SILICEO-NEXUS</span>
            <span class="tag">GATEWAY V0.1.0</span>
        </div>

        <div class="search-box">
            <span>🔍</span>
            <input type="text" id="global-search" placeholder="Search Models, Keys, Providers..." oninput="filterCardsAndCatalog()">
        </div>

        <div class="top-right">
            <div class="admin-box">
                <span style="color:var(--muted);">🔒 Token:</span>
                <input type="password" id="admin-token" placeholder="NEXUS_ADMIN_TOKEN" oninput="localStorage.setItem('nexus_admin_token', this.value)">
            </div>
            <div class="node-status">
                <span class="dot"></span>
                <span>99.8% Gateway Uptime</span>
            </div>
        </div>
    </div>

    <!-- Stats Row -->
    <div class="stats-grid">
        <div class="stat-card">
            <div class="stat-label">Configured Providers</div>
            <div class="stat-val" id="stat-count">17 <span class="sub">Active Pool</span></div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Active Models</div>
            <div class="stat-val" id="stat-models">450 <span class="sub">• Live Discovering</span></div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Masked API Keys</div>
            <div class="stat-val" id="stat-keys">17 <span class="sub" style="color:var(--primary);">100% Masked</span></div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Gateway Uptime</div>
            <div class="stat-val" style="color:var(--success);">99.8% <span class="sub">30 Days</span></div>
        </div>
    </div>

    <!-- Navigation Tabs & Add Action -->
    <div class="nav-tabs">
        <div class="tabs-group" id="tabs-bar">
            <button class="tab-btn active" id="tab-btn-providers" onclick="switchTab('providers')">⚡ Provider Catalog</button>
            <button class="tab-btn" id="tab-btn-catalog" onclick="switchTab('cat-all')">📚 Model Gateway (450+)</button>
            <button class="tab-btn" id="tab-btn-telemetry" onclick="switchTab('telemetry')">📈 Live Telemetry</button>
        </div>
        <button class="btn-add" onclick="openAddModal()">➕ Registra Provider</button>
    </div>

    <!-- Main View: Provider Cards Grid -->
    <div id="view-providers" class="cards-grid">
        <!-- Rendered dynamically via JavaScript -->
    </div>

    <!-- Alternative View: Model Catalog Table -->
    <div id="view-catalog" class="table-container" style="display:none;">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">
            <h3 id="catalog-title" style="color:var(--primary); font-size:1.1rem;">📚 Catalogo Modelli Sincronizzato</h3>
            <div style="display:flex; gap:10px;">
                <input type="text" id="catalog-search" placeholder="Filtra modelli..." oninput="filterCatalog()" style="width:220px; padding:6px 12px;">
                <select id="catalog-filter" onchange="filterCatalog()" style="width:140px; padding:6px 12px;">
                    <option value="all">Tutti i tipi</option>
                    <option value="free">Solo Gratuiti</option>
                </select>
                <button class="btn-add" style="padding:6px 12px; font-size:0.8rem;" onclick="syncCatalog()">🔄 Sincronizza Ora</button>
            </div>
        </div>
        <table>
            <thead>
                <tr>
                    <th>Modello ID</th>
                    <th>Sorgente</th>
                    <th>Costo Prompt (1M)</th>
                    <th>Costo Completion (1M)</th>
                    <th>Contesto</th>
                    <th>Stato</th>
                    <th>Azione</th>
                </tr>
            </thead>
            <tbody id="catalog-body">
                <!-- Rendered dynamically -->
            </tbody>
        </table>
    </div>

    <!-- Dedicated View: Real-Time Telemetry & Hardware Load -->
    <div id="view-telemetry" style="display:none;">
        <div class="telemetry-panel">
            <div class="telemetry-header">
                <div class="telemetry-title">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
                    <span>LIVE HARDWARE & GATEWAY TELEMETRY DASHBOARD</span>
                </div>
                <div style="display:flex; align-items:center; gap:8px; font-size:0.75rem; color:var(--muted); font-family:monospace;">
                    <span class="live-pulse"></span> SYSTEM POLLING (2s)
                </div>
            </div>
            <div class="telemetry-grid">
                <div class="telemetry-box">
                    <div class="telemetry-label">
                        <span>RTX 2070 GPU & SYSTEM LOAD</span>
                        <strong id="telemetry-gpu-pct" style="color:var(--primary);">45%</strong>
                    </div>
                    <div class="gauge-bar">
                        <div class="gauge-fill" id="telemetry-gpu-bar" style="width: 45%;"></div>
                    </div>
                    <div style="font-size:0.75rem; color:var(--muted); margin-top:10px; font-family:monospace; display:flex; justify-content:space-between;">
                        <span>Node: beellama-tailscale-2070</span>
                        <span id="telemetry-mem-val">RAM: 42%</span>
                    </div>
                </div>
                <div class="telemetry-box">
                    <div class="telemetry-label">
                        <span>GATEWAY LATENCY SPARKLINE (MS)</span>
                        <strong id="telemetry-latency-val" style="color:var(--success);">14 ms</strong>
                    </div>
                    <div class="sparkline-box" id="telemetry-sparkline" style="height: 48px;">
                        <div class="spark-bar" style="height: 40%;"></div>
                        <div class="spark-bar" style="height: 60%;"></div>
                        <div class="spark-bar" style="height: 35%;"></div>
                        <div class="spark-bar" style="height: 50%;"></div>
                        <div class="spark-bar" style="height: 75%;"></div>
                        <div class="spark-bar" style="height: 45%;"></div>
                        <div class="spark-bar" style="height: 90%;"></div>
                        <div class="spark-bar active" style="height: 55%;"></div>
                    </div>
                </div>
            </div>
        </div>
    </div>
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">
            <h3 id="catalog-title" style="color:var(--primary); font-size:1.1rem;">📚 Catalogo Modelli Sincronizzato</h3>
            <div style="display:flex; gap:10px;">
                <input type="text" id="catalog-search" placeholder="Filtra modelli..." oninput="filterCatalog()" style="width:220px; padding:6px 12px;">
                <select id="catalog-filter" onchange="filterCatalog()" style="width:140px; padding:6px 12px;">
                    <option value="all">Tutti i tipi</option>
                    <option value="free">Solo Gratuiti</option>
                </select>
                <button class="btn-add" style="padding:6px 12px; font-size:0.8rem;" onclick="syncCatalog()">🔄 Sincronizza Ora</button>
            </div>
        </div>
        <table>
            <thead>
                <tr>
                    <th>Modello ID</th>
                    <th>Sorgente</th>
                    <th>Costo Prompt (1M)</th>
                    <th>Costo Completion (1M)</th>
                    <th>Contesto</th>
                    <th>Stato</th>
                    <th>Azione</th>
                </tr>
            </thead>
            <tbody id="catalog-body">
                <!-- Rendered dynamically -->
            </tbody>
        </table>
    </div>

    <!-- Modal Form: Add / Edit Provider -->
    <div class="modal-overlay" id="form-modal">
        <div class="modal-box">
            <h3 id="form-title">➕ Registra Nuovo Provider
                <span style="cursor:pointer; color:var(--muted); font-size:1.1rem;" onclick="closeFormModal()">✕</span>
            </h3>
            <form id="provider-form" onsubmit="saveProvider(event)">
                <input type="hidden" id="p-id">
                
                <div class="form-group">
                    <label>⚡ Seleziona Preset Provider (Compilazione Automatica)</label>
                    <select id="p-preset" onchange="applyPresetProvider(this.value)">
                        <option value="">-- Seleziona un Preset o Inserisci Manualmente --</option>
                        <option value="groq">⚡ Groq Cloud (Llama 3.3 70B, Mixtral)</option>
                        <option value="google">♊ Google AI Studio (Gemini 2.5 Flash / Pro)</option>
                        <option value="deepseek">🧠 DeepSeek (V3, R1)</option>
                        <option value="nvidia">🟢 NVIDIA NIM (Llama 3 70B, Nemotron)</option>
                        <option value="qwen">🐉 Alibaba Cloud / Qwen (DashScope)</option>
                        <option value="anthropic">🎨 Anthropic (Claude 3.5 Sonnet)</option>
                        <option value="openai">🤖 OpenAI (GPT-4o, GPT-4o-mini)</option>
                        <option value="aws">☁️ AWS Bedrock / Mantle Proxy</option>
                        <option value="inception">🔥 Inception / Fireworks AI</option>
                        <option value="agnes">🕊️ Agnes AI Singapore Cloud (Omni-Modal)</option>
                        <option value="mistral">🌪️ Mistral AI (Mistral Large, Codestral)</option>
                        <option value="together">🤝 Together AI</option>
                        <option value="perplexity">🔍 Perplexity AI (Sonar Reasoning)</option>
                        <option value="cerebras">⚡ Cerebras AI (Ultra Fast)</option>
                        <option value="sambanova">🟧 SambaNova Systems (Llama 3 405B)</option>
                        <option value="openrouter">🪐 OpenRouter Network (Free Pool)</option>
                        <option value="ollama_local">🏠 Ollama Local (Node RTX 2070)</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>Nome Identificativo Provider</label>
                    <input type="text" id="p-name" placeholder="es. groq-fast-pool" required>
                </div>

                <div class="form-group">
                    <label>Endpoint Base URL (API OpenAI-Compatibile)</label>
                    <input type="text" id="p-url" placeholder="https://api.groq.com/openai/v1" required>
                </div>

                <div class="form-group">
                    <label>API Key (Auto-Detect dal formato o Inserimento Manuale)</label>
                    <input type="password" id="p-key" placeholder="Incolla la tua API Key qui..." oninput="detectProviderFromKey(this.value)">
                </div>

                <div class="form-group">
                    <label>Modello Target Predestinato</label>
                    <div style="display:flex; gap:8px;">
                        <input type="text" id="p-model" placeholder="qwen/qwen-2.5-coder-32b:free" required>
                        <button type="button" style="width:160px; background:var(--card-border); color:var(--primary); font-size:0.8rem;" onclick="fetchLiveModelsFromEndpoint()">🔍 Rileva Modelli</button>
                    </div>
                    <select id="p-model-select" style="display:none; margin-top:8px;" onchange="if(this.value) document.getElementById('p-model').value = this.value;"></select>
                </div>

                <div style="display:grid; grid-template-columns: 1fr 1fr; gap:12px;">
                    <div class="form-group">
                        <label>Tier di Costo</label>
                        <select id="p-tier">
                            <option value="free">Free (Gratuito)</option>
                            <option value="paid">Paid (A Pagamento)</option>
                            <option value="local">Local (Nodo Locale GPU)</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Priorità Cascata (1 = Max Priorità)</label>
                        <input type="number" id="p-priority" value="10" min="1" max="999">
                    </div>
                </div>

                <div class="form-group">
                    <label>Tag Inserimento (separati da virgola)</label>
                    <input type="text" id="p-tags" placeholder="chitchat, coding, reasoning, fast">
                </div>

                <div style="display:flex; justify-content:flex-end; gap:10px; margin-top:20px;">
                    <button type="button" style="background:transparent; color:var(--muted); border:1px solid var(--card-border);" onclick="closeFormModal()">Annulla</button>
                    <button type="submit" class="btn-add" id="btn-save">Salva Provider nel Nexus</button>
                </div>
            </form>
        </div>
    </div>

    <!-- Modal Output Test Connectivity -->
    <div class="modal-overlay" id="test-modal">
        <div class="modal-box">
            <h3>🧪 Esito Test Connettività Provider
                <span style="cursor:pointer; color:var(--muted); font-size:1.1rem;" onclick="closeTestModal()">✕</span>
            </h3>
            <div id="test-content"></div>
            <div style="text-align:right; margin-top:20px;">
                <button type="button" class="btn-add" onclick="closeTestModal()">Chiudi</button>
            </div>
        </div>
    </div>

    <script>
        let loadedProvidersList = [];
        let fullCatalog = [];
        let catalogProvidersMeta = [];
        let activeTabKey = 'providers';
        let currentCatalogSource = 'all';

        const PROVIDER_SVGS = {
            groq: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm1 14.5a4.5 4.5 0 1 1 3.18-7.68"/><circle cx="12" cy="12" r="2.5" fill="currentColor"/></svg>`,
            google: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M12 0L14.59 9.41L24 12L14.59 14.59L12 24L9.41 14.59L0 12L9.41 9.41L12 0Z"/></svg>`,
            anthropic: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M13.8 3h-3.6L4.5 21h3.7l1.3-4h5l1.3 4h3.7L13.8 3zm-3.1 11l1.7-5.2 1.7 5.2h-3.4z"/></svg>`,
            deepseek: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3c-4.97 0-9 4.03-9 9 0 2.12.74 4.07 1.97 5.61L3 21l3.5-.95A8.94 8.94 0 0 0 12 21c4.97 0 9-4.03 9-9s-4.03-9-9-9z"/><path d="M9 11a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zm6 0a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z"/></svg>`,
            nvidia: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M10.15 8.1c1.3.4 2.1 1.5 2.1 2.8v3.2c0 .9-.5 1.8-1.3 2.2-1.3.6-2.9.2-3.7-.9L3 10.2l-1 3.5c.9 2.5 3.3 4.3 6.1 4.3 3.6 0 6.5-2.9 6.5-6.5V9.4c0-2.8-1.8-5.3-4.4-6.1L10.15 8.1zM12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg>`,
            agnes: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 18c6-6 12-4 18-12M3 18c4-1 9-5 9-11M3 18h18"/></svg>`,
            openai: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 6v12M6 12h12"/></svg>`,
            qwen: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>`,
            mistral: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M4 4h4v16H4V4zm6 0h4v16h-4V4zm6 0h4v16h-4V4z"/></svg>`,
            perplexity: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="11" cy="11" r="7"/><path d="M21 21l-4.35-4.35M11 8v6M8 11h6"/></svg>`,
            cerebras: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm-2 14.5l-4-4 1.41-1.41L10 13.67l6.59-6.59L18 8.5l-8 8z"/></svg>`,
            sambanova: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><rect x="3" y="3" width="8" height="8" rx="2"/><rect x="13" y="3" width="8" height="8" rx="2"/><rect x="3" y="13" width="8" height="8" rx="2"/><rect x="13" y="13" width="8" height="8" rx="2"/></svg>`,
            together: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><circle cx="8" cy="12" r="5"/><circle cx="16" cy="12" r="5"/></svg>`,
            inception: `<svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor"><path d="M12 2L2 22h20L12 2zm0 4l6.5 13h-13L12 6z"/></svg>`,
            openrouter: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="12" rx="9" ry="4" transform="rotate(-30 12 12)"/><circle cx="12" cy="12" r="3" fill="currentColor"/></svg>`,
            ollama_local: `<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="4"/><path d="M9 9h6M9 12h6M9 15h4"/></svg>`
        };

        const PROVIDER_COLORS = {
            'groq': '#f97316',
            'google': '#818cf8',
            'anthropic': '#f43f5e',
            'deepseek': '#8b5cf6',
            'nvidia': '#10b981',
            'agnes': '#10b981',
            'openai': '#10b981',
            'openrouter': '#38bdf8',
            'perplexity': '#06b6d4',
            'mistral': '#f59e0b',
            'together': '#14b8a6',
            'cerebras': '#ec4899',
            'sambanova': '#ef4444',
            'inception': '#a855f7',
            'ollama_local': '#3b82f6'
        };

        const PRESETS = {
            groq: { key: "groq", name: "groq-free-pool", url: "https://api.groq.com/openai/v1", tier: "free", priority: 1, tags: "chitchat, fast, groq_free", default_model: "llama-3.3-70b-versatile" },
            google: { key: "google", name: "gemini-free-tier", url: "https://generativelanguage.googleapis.com/v1beta/openai", tier: "free", priority: 1, tags: "chitchat, coding, fast, google_free", default_model: "gemini-2.5-flash" },
            deepseek: { key: "deepseek", name: "deepseek-cloud", url: "https://api.deepseek.com/v1", tier: "paid", priority: 10, tags: "coding, reasoning, deepseek_r1", default_model: "deepseek-reasoner" },
            nvidia: { key: "nvidia", name: "nvidia-nim-cloud", url: "https://integrate.api.nvidia.com/v1", tier: "free", priority: 2, tags: "coding, reasoning, nvidia_nim", default_model: "meta/llama-3.3-70b-instruct" },
            qwen: { key: "qwen", name: "alibaba-qwen-dashscope", url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1", tier: "free", priority: 3, tags: "coding, reasoning, qwen_coder", default_model: "qwen2.5-coder-32b-instruct" },
            anthropic: { key: "anthropic", name: "anthropic-claude", url: "https://api.anthropic.com/v1", tier: "paid", priority: 20, tags: "coding, reasoning, vision", default_model: "claude-3-5-sonnet-20241022" },
            openai: { key: "openai", name: "openai-official", url: "https://api.openai.com/v1", tier: "paid", priority: 20, tags: "coding, reasoning, tool_supported", default_model: "gpt-4o" },
            aws: { key: "aws", name: "aws-bedrock-mantle", url: "http://localhost:3001/v1", tier: "local", priority: 5, tags: "aws, mantle_proxy", default_model: "bedrock-claude-3.5" },
            inception: { key: "inception", name: "fireworks-inception", url: "https://api.fireworks.ai/inference/v1", tier: "free", priority: 4, tags: "fireworks, inception", default_model: "accounts/fireworks/models/deepseek-r1" },
            agnes: { key: "agnes", name: "agnes-ai-singapore", url: "https://apihub.agnes-ai.com/v1", tier: "free", priority: 1, tags: "agnes_ai, omni_modal, singapore_cloud", default_model: "agnes-v1" },
            mistral: { key: "mistral", name: "mistral-ai-cloud", url: "https://api.mistral.ai/v1", tier: "free", priority: 5, tags: "mistral, codestral", default_model: "codestral-latest" },
            together: { key: "together", name: "together-ai", url: "https://api.together.xyz/v1", tier: "free", priority: 5, tags: "together, llama3", default_model: "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo" },
            perplexity: { key: "perplexity", name: "perplexity-sonar", url: "https://api.perplexity.ai", tier: "paid", priority: 10, tags: "search, reasoning, perplexity", default_model: "sonar-reasoning" },
            cerebras: { key: "cerebras", name: "cerebras-fast", url: "https://api.cerebras.ai/v1", tier: "free", priority: 1, tags: "ultra_fast, cerebras", default_model: "llama3.1-70b" },
            sambanova: { key: "sambanova", name: "sambanova-cloud", url: "https://api.sambanova.ai/v1", tier: "free", priority: 2, tags: "sambanova, llama405b", default_model: "Meta-Llama-3.3-70B-Instruct" },
            openrouter: { key: "openrouter", name: "openrouter-free-pool", url: "https://openrouter.ai/api/v1", tier: "free", priority: 2, tags: "chitchat, coding, openrouter_pool", default_model: "qwen/qwen-2.5-coder-32b:free" },
            ollama_local: { key: "ollama_local", name: "ollama-local-gpu", url: "http://100.98.20.76:8080/v1", tier: "local", priority: 1, tags: "rtx_2070, local_gpu, tailscale", default_model: "qwen2.5-coder:32b" }
        };

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

        function openAddModal(isEdit = false) {
            if (!isEdit) {
                resetForm();
            }
            document.getElementById('form-modal').style.display = 'flex';
        }

        function closeFormModal() {
            document.getElementById('form-modal').style.display = 'none';
        }

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

            const editId = document.getElementById('p-id').value;
            const provName = document.getElementById('p-name').value;

            try {
                const res = await fetch('/providers/fetch_models', {
                    method: 'POST',
                    headers: getAuthHeaders(),
                    body: JSON.stringify({
                        base_url: baseUrl,
                        api_key: apiKey || null,
                        provider_key: provKey,
                        provider_id: editId ? parseInt(editId) : null,
                        provider_name: provName || null
                    })
                });

                let data;
                const contentType = res.headers.get('content-type') || '';
                if (contentType.includes('application/json')) {
                    data = await res.json();
                } else {
                    const textErr = await res.text();
                    data = { success: false, error: textErr };
                }

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
                alert("⚠️ Errore di connettività durante il rilevamento: " + e.message);
                modelInput.placeholder = "qwen/qwen-2.5-coder-32b:free";
            }
        }

        async function loadProviders() {
            try {
                const res = await fetch('/providers');
                const data = await res.json();
                loadedProvidersList = data.providers || [];
                const container = document.getElementById('view-providers');
                container.innerHTML = '';

                document.getElementById('stat-count').innerText = loadedProvidersList.length;

                if (loadedProvidersList.length === 0) {
                    container.innerHTML = '<div style="color:var(--muted); grid-column: 1/-1; text-align:center; padding:40px;">Nessun provider configurato. Clicca su <strong>➕ Registra Provider</strong> per iniziare!</div>';
                    return;
                }

                loadedProvidersList.forEach(p => {
                    const isCooldown = p.cooldown_until && new Date(p.cooldown_until) > new Date();
                    const statusPill = isCooldown
                        ? `<span class="status-pill cooldown">⏳ Cooldown</span>`
                        : (p.enabled ? `<span class="status-pill active"><span class="dot"></span> Active</span>` : `<span class="status-pill" style="background:rgba(255,255,255,0.05); color:var(--muted);">Disabled</span>`);

                    const matchedPreset = Object.keys(PRESETS).find(k => p.name.toLowerCase().includes(k) || p.base_url.toLowerCase().includes(k)) || 'openai';
                    const iconColor = PROVIDER_COLORS[matchedPreset] || '#38bdf8';
                    const svgIcon = PROVIDER_SVGS[matchedPreset] || `<span style="font-weight:bold; font-size:1.1rem;">${p.name.charAt(0).toUpperCase()}</span>`;

                    const maskedKey = p.api_key ? p.api_key : 'Nessuna Chiave (Pubblico)';
                    const tagsHtml = (p.tags || []).map(t => `<span style="background:var(--bg); border:1px solid var(--card-border); padding:2px 6px; border-radius:4px; font-size:0.7rem;">${t}</span>`).join(' ');

                    container.innerHTML += `
                        <div class="provider-card">
                            <div class="card-header">
                                <div class="provider-brand">
                                    <div class="provider-icon" style="border-color:${iconColor}; color:${iconColor}; box-shadow: 0 0 12px ${iconColor}44;">${svgIcon}</div>
                                    <div>
                                        <div class="provider-name">${p.name}</div>
                                        <div class="provider-sub">Target: ${p.model}</div>
                                    </div>
                                </div>
                                ${statusPill}
                            </div>
                            <div class="card-body">
                                <div style="display:flex; justify-content:space-between; margin-bottom:6px;">
                                    <span>Tier: <strong style="color:var(--text);">${p.tier.toUpperCase()}</strong></span>
                                    <span>Priorità: <strong style="color:var(--primary);">P${p.priority}</strong></span>
                                </div>
                                <div class="key-badge">
                                    <span>🔑 API Key:</span>
                                    <strong>${maskedKey}</strong>
                                </div>
                                <div style="margin-top:8px; display:flex; gap:4px; flex-wrap:wrap;">
                                    ${tagsHtml}
                                </div>
                            </div>
                            <div class="card-actions">
                                <button class="btn-test" onclick="testProvider(${p.id})">🧪 Test</button>
                                <button onclick="editProvider(${p.id})">✏️ Modifica</button>
                                <button class="btn-danger" onclick="deleteProvider(${p.id})">🗑️ Elimina</button>
                            </div>
                        </div>
                    `;
                });
            } catch(e) {
                console.error("Errore caricamento provider:", e);
            }
        }

        function editProvider(id) {
            const p = loadedProvidersList.find(x => x.id === id);
            if (!p) return;

            openAddModal(true);

            document.getElementById('p-id').value = p.id;
            document.getElementById('p-name').value = p.name;
            document.getElementById('p-url').value = p.base_url;
            document.getElementById('p-key').value = p.api_key || '';
            document.getElementById('p-model').value = p.model;
            document.getElementById('p-tier').value = p.tier;
            document.getElementById('p-priority').value = p.priority;
            document.getElementById('p-tags').value = (p.tags || []).join(', ');
            
            const matchedPreset = Object.keys(PRESETS).find(k => p.name.toLowerCase().includes(k) || p.base_url.toLowerCase().includes(k));
            if (matchedPreset) {
                document.getElementById('p-preset').value = matchedPreset;
            } else {
                document.getElementById('p-preset').value = '';
            }

            document.getElementById('form-title').innerText = `✏️ Modifica '${p.name}'`;
            document.getElementById('btn-save').innerText = 'Aggiorna Provider';
        }

        function resetForm() {
            document.getElementById('provider-form').reset();
            document.getElementById('p-id').value = '';
            document.getElementById('p-preset').value = '';
            document.getElementById('p-model-select').style.display = 'none';
            document.getElementById('form-title').innerText = '➕ Registra Nuovo Provider';
            document.getElementById('btn-save').innerText = 'Salva Provider nel Nexus';
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

            closeFormModal();
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
                        <div style="background:rgba(16,185,129,0.1); border:1px solid var(--success); padding:16px; border-radius:10px;">
                            <p style="color:var(--success); font-weight:bold; margin-bottom:5px;">✅ Test Riuscito! (${data.latency_ms} ms)</p>
                            <p style="font-size:0.85rem;"><strong>Provider:</strong> ${data.provider_name}</p>
                            <p style="font-size:0.85rem;"><strong>Modello:</strong> <code>${data.model_used}</code></p>
                            <hr style="border-color:var(--card-border); margin:10px 0;">
                            <p style="font-size:0.85rem; color:var(--text);"><strong>Risposta:</strong> "${data.content}"</p>
                        </div>
                    `;
                } else {
                    document.getElementById('test-content').innerHTML = `
                        <div style="background:rgba(239,68,68,0.1); border:1px solid var(--danger); padding:16px; border-radius:10px;">
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

        function switchTab(tabKey) {
            activeTabKey = tabKey;

            const btnProviders = document.getElementById('tab-btn-providers');
            const btnCatalog = document.getElementById('tab-btn-catalog');
            const btnTelemetry = document.getElementById('tab-btn-telemetry');

            const viewProviders = document.getElementById('view-providers');
            const viewCatalog = document.getElementById('view-catalog');
            const viewTelemetry = document.getElementById('view-telemetry');

            btnProviders.classList.remove('active');
            btnCatalog.classList.remove('active');
            btnTelemetry.classList.remove('active');

            viewProviders.style.display = 'none';
            viewCatalog.style.display = 'none';
            viewTelemetry.style.display = 'none';

            if (tabKey === 'providers') {
                btnProviders.classList.add('active');
                viewProviders.style.display = 'grid';
            } else if (tabKey === 'telemetry') {
                btnTelemetry.classList.add('active');
                viewTelemetry.style.display = 'block';
            } else {
                btnCatalog.classList.add('active');
                viewCatalog.style.display = 'block';

                if (tabKey.startsWith('cat-') && tabKey !== 'cat-all') {
                    currentCatalogSource = tabKey.replace('cat-', '');
                } else {
                    currentCatalogSource = 'all';
                }
                filterCatalog();
            }
        }

        async function loadCatalog() {
            try {
                const res = await fetch('/catalog');
                const data = await res.json();
                fullCatalog = data.catalog || [];
                catalogProvidersMeta = data.catalog_providers || [];

                document.getElementById('stat-models').innerHTML = `${fullCatalog.length} <span class="sub">• Synchronized</span>`;
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
                tbody.innerHTML = '<tr><td colspan="7" style="color:var(--muted); text-align:center; padding:20px;">Nessun modello trovato per i filtri selezionati.</td></tr>';
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
                            <button style="background:var(--card-border); color:var(--text); padding:3px 8px; font-size:0.75rem; border-radius:4px; cursor:pointer;" onclick="useModelInForm('${m.model_id}', '${m.provider_name}')">➕ Usa</button>
                        </td>
                    </tr>
                `;
            });
        }

        function filterCardsAndCatalog() {
            const query = document.getElementById('global-search').value.toLowerCase();
            const cards = document.querySelectorAll('.provider-card');
            cards.forEach(c => {
                const text = c.innerText.toLowerCase();
                c.style.display = text.includes(query) ? 'block' : 'none';
            });
            if (document.getElementById('catalog-search')) {
                document.getElementById('catalog-search').value = query;
                filterCatalog();
            }
        }

        function useModelInForm(modelId, providerName) {
            switchTab('providers');
            openAddModal();
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

        let sparklineHistory = [40, 55, 30, 60, 45, 80, 50, 65];

        async function loadLiveStats() {
            try {
                const res = await fetch('/stats');
                const data = await res.json();
                if (data) {
                    if (data.gpu_utilization_pct !== undefined) {
                        document.getElementById('telemetry-gpu-pct').innerText = data.gpu_utilization_pct + '%';
                        document.getElementById('telemetry-gpu-bar').style.width = data.gpu_utilization_pct + '%';
                    }
                    if (data.system_memory_used_pct !== undefined) {
                        document.getElementById('telemetry-mem-val').innerText = 'RAM: ' + data.system_memory_used_pct + '%';
                    }
                    if (data.last_latency_ms !== undefined) {
                        document.getElementById('telemetry-latency-val').innerText = data.last_latency_ms + ' ms';
                        sparklineHistory.shift();
                        sparklineHistory.push(Math.min(100, Math.max(15, data.last_latency_ms)));

                        const sparkBox = document.getElementById('telemetry-sparkline');
                        if (sparkBox) {
                            sparkBox.innerHTML = '';
                            sparklineHistory.forEach((val, idx) => {
                                const isLast = idx === sparklineHistory.length - 1;
                                const cls = isLast ? 'spark-bar active' : (val > 60 ? 'spark-bar high' : 'spark-bar');
                                sparkBox.innerHTML += `<div class="${cls}" style="height:${val}%;"></div>`;
                            });
                        }
                    }
                }
            } catch(e) {
                console.error("Errore telemetria:", e);
            }
        }

        loadProviders();
        loadCatalog();
        loadLiveStats();
        setInterval(loadLiveStats, 2000);
    </script>
</body>
</html>"##)
}
