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
        .node-status { display: flex; gap: 10px; align-items: center; font-size: 0.85rem; padding: 8px 12px; background: rgba(56, 189, 248, 0.1); border-radius: 8px; border: 1px solid rgba(56, 189, 248, 0.2); }
        .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--success); display: inline-block; }
        .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; margin-bottom: 20px; }
        .stat-card { background: var(--panel); border: 1px solid var(--border); padding: 15px; border-radius: 10px; text-align: center; }
        .stat-val { font-size: 1.5rem; font-weight: bold; color: var(--primary); margin-top: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>💎 Siliceo-Nexus <span style="font-size:0.8rem; color:var(--muted); font-weight:normal;">v0.1.0 (Port :8082)</span></h1>
        <div class="node-status">
            <span class="dot"></span>
            <span>Tailscale GPU Node: <strong>100.98.20.76 (:8080)</strong></span>
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

    <div class="tabs">
        <button class="tab-btn active" onclick="showTab('tab-providers')">⚡ Providers & Pool Stack</button>
        <button class="tab-btn" onclick="showTab('tab-catalog')">📚 Catalogo Modelli (394)</button>
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
                <h2>➕ Aggiungi / Modifica Provider</h2>
                <form id="provider-form" onsubmit="saveProvider(event)">
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
                        <textarea id="p-key" rows="2" placeholder="KEY_1, KEY_2, KEY_3 (Multi-Key Round-Robin)"></textarea>
                    </div>
                    <div class="form-group">
                        <label>Modello Target</label>
                        <input type="text" id="p-model" placeholder="qwen/qwen-2.5-coder-32b:free" required>
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
                    <button type="submit">Salva Provider nel Nexus</button>
                </form>
            </div>
        </div>
    </div>

    <!-- TAB 2: CATALOGO MODELLI -->
    <div id="tab-catalog" class="tab-content">
        <div class="card">
            <h2>📚 Catalogo Ufficiale Modelli OpenRouter (Sincronizzato 24h) 
                <button onclick="syncCatalog()">🔄 Sincronizza Ora</button>
            </h2>
            <div style="display:flex; gap:15px; margin-bottom:15px;">
                <input type="text" id="catalog-search" placeholder="🔍 Cerca modello (es. qwen, llama, deepseek, free)..." oninput="filterCatalog()">
                <select id="catalog-filter" onchange="filterCatalog()" style="width:200px;">
                    <option value="all">Tutti i Modelli</option>
                    <option value="free" selected>Solo 100% Free ($0.00)</option>
                </select>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>ID Modello</th>
                        <th>Costo Prompt / 1M</th>
                        <th>Costo Comp / 1M</th>
                        <th>Contesto</th>
                        <th>Gratuito</th>
                        <th>Azione</th>
                    </tr>
                </thead>
                <tbody id="catalog-body">
                    <tr><td colspan="6" style="color:var(--muted); text-align:center;">Caricamento catalogo...</td></tr>
                </tbody>
            </table>
        </div>
    </div>

    <script>
        let fullCatalog = [];

        function showTab(tabId) {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            event.target.classList.add('active');
            document.getElementById(tabId).classList.add('active');
            if (tabId === 'tab-catalog' && fullCatalog.length === 0) {
                loadCatalog();
            }
        }

        async function loadProviders() {
            try {
                const res = await fetch('/providers');
                const data = await res.json();
                const tbody = document.getElementById('providers-body');
                tbody.innerHTML = '';

                if (!data.providers || data.providers.length === 0) {
                    tbody.innerHTML = '<tr><td colspan="7" style="color:var(--muted); text-align:center;">Nessun provider configurato.</td></tr>';
                    return;
                }

                document.getElementById('stat-count').innerText = data.providers.length;

                data.providers.forEach(p => {
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
                            <td>
                                <button class="btn-secondary" style="padding:4px 8px; font-size:0.75rem;" onclick="testProvider('${p.name}')">🧪 Test</button>
                                <button class="btn-danger" style="padding:4px 8px; font-size:0.75rem;" onclick="deleteProvider(${p.id})">🗑️</button>
                            </td>
                        </tr>
                    `;
                });
            } catch(e) {
                console.error("Errore caricamento provider:", e);
            }
        }

        async function saveProvider(e) {
            e.preventDefault();
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

            await fetch('/providers', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });

            document.getElementById('provider-form').reset();
            loadProviders();
        }

        async function deleteProvider(id) {
            if (confirm("Vuoi rimuovere questo provider da Siliceo-Nexus?")) {
                await fetch(`/providers/${id}`, { method: 'DELETE' });
                loadProviders();
            }
        }

        async function testProvider(name) {
            alert(`🧪 Inizio test di completamento live per '${name}' via Siliceo-Nexus...`);
        }

        async function loadCatalog() {
            try {
                const res = await fetch('/catalog');
                const data = await res.json();
                fullCatalog = data.catalog || [];
                document.getElementById('stat-models').innerText = data.count || fullCatalog.length;
                filterCatalog();
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
                return matchesQuery && matchesFilter;
            });

            if (filtered.length === 0) {
                tbody.innerHTML = '<tr><td colspan="6" style="color:var(--muted); text-align:center;">Nessun modello trovato nel catalogo.</td></tr>';
                return;
            }

            filtered.slice(0, 100).forEach(m => {
                const isFreeBadge = m.is_free ? '<span class="badge badge-free">FREE</span>' : '<span class="badge badge-paid">PAID</span>';
                const ctx = (m.context_length / 1024).toFixed(0) + 'k';
                tbody.innerHTML += `
                    <tr>
                        <td><code>${m.model_id}</code></td>
                        <td>$${m.prompt_cost_per_1m.toFixed(4)}</td>
                        <td>$${m.completion_cost_per_1m.toFixed(4)}</td>
                        <td>${ctx} tokens</td>
                        <td>${isFreeBadge}</td>
                        <td>
                            <button class="btn-secondary" style="padding:3px 8px; font-size:0.75rem;" onclick="useModelInForm('${m.model_id}')">➕ Usa</button>
                        </td>
                    </tr>
                `;
            });
        }

        function useModelInForm(modelId) {
            showTab('tab-providers');
            document.getElementById('p-model').value = modelId;
            if (modelId.includes(':free')) {
                document.getElementById('p-tier').value = 'free';
            }
        }

        async function syncCatalog() {
            alert("🔄 Sincronizzazione catalogo OpenRouter avviata...");
            await fetch('/catalog/sync', { method: 'POST' });
            loadCatalog();
        }

        loadProviders();
    </script>
</body>
</html>"#)
}
