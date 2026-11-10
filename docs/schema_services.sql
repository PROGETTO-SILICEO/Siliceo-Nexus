-- Nexus — schema servizi/nodi/lease
-- NON eseguire sul DB di produzione finché non è approvato.
-- Idempotente: può essere applicato a una copia o a un DB di test.

CREATE TABLE IF NOT EXISTS nodes (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    base_url        TEXT NOT NULL,
    auth_token      TEXT,
    tailscale_ip    TEXT,
    enabled         INTEGER NOT NULL DEFAULT 1,
    last_seen       TEXT,
    last_error      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS services (
    id              TEXT PRIMARY KEY,
    node_id         TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL CHECK (kind IN ('llm','image','voice','stt')),
    name            TEXT NOT NULL,
    base_url        TEXT NOT NULL,
    models_json     TEXT NOT NULL DEFAULT '[]',
    tags_json       TEXT NOT NULL DEFAULT '[]',
    priority        INTEGER NOT NULL DEFAULT 100,
    max_ctx         INTEGER,
    vram_mb         INTEGER,
    enabled         INTEGER NOT NULL DEFAULT 1,
    health_url      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_services_node ON services(node_id);
CREATE INDEX IF NOT EXISTS idx_services_kind ON services(kind, enabled, priority);

CREATE TABLE IF NOT EXISTS leases (
    id              TEXT PRIMARY KEY,
    service_id      TEXT NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    model           TEXT,
    acquired_at     TEXT NOT NULL,
    expires_at      TEXT,
    released_at     TEXT,
    status          TEXT NOT NULL CHECK (status IN ('active','released','expired','conflict')),
    reason          TEXT
);

CREATE INDEX IF NOT EXISTS idx_leases_active ON leases(status, service_id);

-- Seed del nodo reale, disabilitato di default: non cambia il comportamento
-- finché NEXUS_SERVICES_ENABLED non è true.
INSERT OR IGNORE INTO nodes (id, name, base_url, tailscale_ip, enabled)
VALUES ('nodo2070', 'Nodo Inferenza RTX 2070', 'http://100.98.20.76:8080', '100.98.20.76', 0);

INSERT OR IGNORE INTO services
    (id, node_id, kind, name, base_url, models_json, tags_json, priority, max_ctx, vram_mb, enabled)
VALUES
    ('llm-9b', 'nodo2070', 'llm', 'Qwen 9B locale',
     'http://100.98.20.76:8080',
     '["Qwen_Qwen3.5-9B-Q4_K_M.gguf"]',
     '["chitchat","reasoning","local"]',
     1, 8192, 6800, 0),
    ('comfyui', 'nodo2070', 'image', 'ComfyUI',
     'http://100.98.20.76:8080',
     '["comfyui"]',
     '["image","local"]',
     10, NULL, 5200, 0),
    ('voice-piper', 'nodo2070', 'voice', 'Piper TTS',
     'http://100.98.20.76:8080',
     '["piper-it-paola"]',
     '["voice","local","cpu"]',
     20, NULL, 0, 0);
