use axum::response::Html;

pub fn render_dashboard() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="it">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>💎 Siliceo-Nexus — Universal Inference Gateway</title>
    <style>
        :root {
            --bg: #0b0f19;
            --panel: #131b2e;
            --border: #1e293b;
            --primary: #38bdf8;
            --accent: #818cf8;
            --success: #34d399;
            --danger: #f87171;
            --text: #f8fafc;
            --muted: #94a3b8;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; }
        body { background: var(--bg); color: var(--text); padding: 20px; line-height: 1.5; }
        .header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 20px; margin-bottom: 20px; }
        .header h1 { font-size: 1.5rem; display: flex; align-items: center; gap: 10px; color: var(--primary); }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px; margin-bottom: 25px; }
        .card { background: var(--panel); border: 1px solid var(--border); border-radius: 12px; padding: 20px; }
        .card h2 { font-size: 1.1rem; color: var(--accent); margin-bottom: 15px; display: flex; justify-content: space-between; align-items: center; }
        table { width: 100%; border-collapse: collapse; margin-top: 10px; }
        th, td { text-align: left; padding: 10px; border-bottom: 1px solid var(--border); font-size: 0.9rem; }
        th { color: var(--muted); font-weight: 500; }
        .badge { padding: 3px 8px; border-radius: 6px; font-size: 0.75rem; font-weight: 600; text-transform: uppercase; }
        .badge-free { background: rgba(52, 211, 153, 0.15); color: var(--success); }
        .badge-paid { background: rgba(248, 113, 113, 0.15); color: var(--danger); }
        .badge-local { background: rgba(56, 189, 248, 0.15); color: var(--primary); }
        .form-group { margin-bottom: 12px; }
        .form-group label { display: block; font-size: 0.85rem; color: var(--muted); margin-bottom: 5px; }
        input, select { width: 100%; padding: 8px 12px; background: var(--bg); border: 1px solid var(--border); border-radius: 6px; color: var(--text); font-size: 0.9rem; }
        button { background: var(--primary); color: #000; border: none; padding: 10px 16px; border-radius: 6px; font-weight: 600; cursor: pointer; transition: opacity 0.2s; }
        button:hover { opacity: 0.9; }
        .node-status { display: flex; gap: 10px; align-items: center; font-size: 0.85rem; padding: 8px 12px; background: rgba(56, 189, 248, 0.1); border-radius: 8px; border: 1px solid rgba(56, 189, 248, 0.2); }
        .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--success); display: inline-block; }
    </style>
</head>
<body>
    <div class="header">
        <h1>💎 Siliceo-Nexus <span style="font-size:0.8rem; color:var(--muted); font-weight:normal;">v0.1.0 (Port :8082)</span></h1>
        <div class="node-status">
            <span class="dot"></span>
            <span>Tailscale Mesh: <strong>nodo-inferenza (100.98.20.76)</strong></span>
        </div>
    </div>

    <div class="grid">
        <div class="card" style="grid-column: span 2;">
            <h2>⚡ Provider Attivi & Pool Stack <button onclick="loadProviders()">🔄 Aggiorna</button></h2>
            <table>
                <thead>
                    <tr>
                        <th>Nome Provider</th>
                        <th>Modello Target</th>
                        <th>Tier</th>
                        <th>Priorità</th>
                        <th>TPM Max</th>
                        <th>Capabilities</th>
                        <th>Stato</th>
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
                    <input type="text" id="p-name" placeholder="es. gemini-free-2" required>
                </div>
                <div class="form-group">
                    <label>Base URL / Endpoint</label>
                    <input type="text" id="p-url" placeholder="http://100.98.20.76:8080" required>
                </div>
                <div class="form-group">
                    <label>API Key (opzionale)</label>
                    <input type="password" id="p-key" placeholder="Bearer Key...">
                </div>
                <div class="form-group">
                    <label>Modello Default</label>
                    <input type="text" id="p-model" placeholder="Qwen3.5-4B-Q6_K.gguf" required>
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
                    <input type="text" id="p-tags" placeholder="chitchat, coding, tool_supported">
                </div>
                <button type="submit">Salva Provider nel Nexus</button>
            </form>
        </div>
    </div>

    <script>
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

                data.providers.forEach(p => {
                    const badgeClass = p.tier === 'local' ? 'badge-local' : (p.tier === 'free' ? 'badge-free' : 'badge-paid');
                    const tags = (p.tags || []).map(t => `<span style="font-size:0.7rem; background:#1e293b; padding:2px 5px; border-radius:4px; margin-right:3px;">${t}</span>`).join('');
                    const status = p.enabled ? '🟢 Attivo' : '🔴 Disabilitato';

                    tbody.innerHTML += `
                        <tr>
                            <td><strong>${p.name}</strong></td>
                            <td><code>${p.model}</code></td>
                            <td><span class="badge ${badgeClass}">${p.tier}</span></td>
                            <td>P${p.priority}</td>
                            <td>${p.tpm_limit.toLocaleString()} TPM</td>
                            <td>${tags}</td>
                            <td>${status}</td>
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
                tpm_limit: 32000,
                rpm_limit: 15,
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

        loadProviders();
    </script>
</body>
</html>"#)
}
