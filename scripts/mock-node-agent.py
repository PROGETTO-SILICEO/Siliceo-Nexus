#!/usr/bin/env python3
"""mock-node-agent.py — finto Node Agent per testare Nexus senza toccare la 2070.

Non carica modelli, non usa GPU, non parla con il nodo. Serve solo a validare
contratto, routing, lease e proxy di Nexus in locale.

Uso:
    python3 scripts/mock-node-agent.py --port 18080
    curl http://127.0.0.1:18080/v1/services
"""

from __future__ import annotations

import argparse
import json
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse

STATE = {
    "active": None,
    "lease_id": None,
    "model": None,
    "since": None,
    "gpu_used_mb": 0,
    "requests": 0,
}


def now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


SERVICES = [
    {
        "id": "llm",
        "kind": "llm",
        "name": "Mock LLM 9B",
        "port": 8086,
        "running": True,
        "models": ["Qwen_Qwen3.5-9B-Q4_K_M.gguf"],
        "vram_mb": 6800,
    },
    {
        "id": "image",
        "kind": "image",
        "name": "Mock ComfyUI",
        "port": 8188,
        "running": True,
        "models": ["comfyui"],
        "vram_mb": 5200,
    },
    {
        "id": "voice",
        "kind": "voice",
        "name": "Mock Voice",
        "port": 1135,
        "running": False,
        "models": ["piper-it-paola"],
        "vram_mb": 0,
    },
    {
        "id": "stt",
        "kind": "stt",
        "name": "Mock STT",
        "port": 1135,
        "running": False,
        "models": ["parakeet"],
        "vram_mb": 0,
    },
]


class Handler(BaseHTTPRequestHandler):
    server_version = "MockNodeAgent/0.1"

    def log_message(self, fmt, *args):
        print(f"[mock-node] {self.address_string()} {fmt % args}", flush=True)

    def _json(self, code: int, obj: dict):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _read_json(self) -> dict:
        n = int(self.headers.get("Content-Length", "0"))
        if not n:
            return {}
        try:
            return json.loads(self.rfile.read(n))
        except Exception:
            return {}

    def do_GET(self):
        path = urlparse(self.path).path
        if path == "/health":
            return self._json(200, {
                "status": "ok",
                "service": "mock-node-agent",
                "active": STATE["active"],
                "active_model": STATE["model"],
                "lease_id": STATE["lease_id"],
                "gpu": {"total_mb": 8192, "used_mb": STATE["gpu_used_mb"], "free_mb": 8192 - STATE["gpu_used_mb"]},
            })
        if path == "/v1/services":
            return self._json(200, {
                "node": "mock-2070",
                "gpu": {"name": "Mock RTX 2070", "total_mb": 8192, "used_mb": STATE["gpu_used_mb"], "free_mb": 8192 - STATE["gpu_used_mb"]},
                "active": STATE["active"],
                "lease_id": STATE["lease_id"],
                "services": [
                    dict(s, running=(s["id"] == STATE["active"] or s["kind"] == "image"))
                    for s in SERVICES
                ],
            })
        if path == "/v1/services/llm":
            return self._json(200, next(s for s in SERVICES if s["id"] == "llm"))
        return self._json(404, {"ok": False, "error": "not_found"})

    def do_POST(self):
        path = urlparse(self.path).path
        body = self._read_json()

        if path == "/v1/lease":
            service = body.get("service")
            model = body.get("model")
            if service not in {s["id"] for s in SERVICES}:
                return self._json(400, {"ok": False, "error": "unknown_service"})
            # conflitto simulato: un processo non registrato tiene la GPU
            if body.get("simulate_conflict"):
                return self._json(409, {
                    "ok": False,
                    "error": "vram_conflict",
                    "holder": {"pid": 598366, "name": "lfm3-venv/bin/python", "mem_mb": 3416},
                })
            lease_id = str(uuid.uuid4())
            STATE.update(active=service, lease_id=lease_id, model=model, since=now())
            STATE["gpu_used_mb"] = 6800 if service == "llm" else 5200 if service == "image" else 0
            return self._json(200, {
                "ok": True,
                "lease_id": lease_id,
                "service": service,
                "model": model,
                "endpoint": f"mock://{service}",
                "expires_at": "2026-11-03T21:30:00Z",
                "switched": True,
            })

        if path == "/v1/release":
            lease_id = body.get("lease_id")
            if lease_id and lease_id != STATE["lease_id"]:
                return self._json(409, {"ok": False, "error": "lease_mismatch"})
            STATE.update(active=None, lease_id=None, model=None, since=None, gpu_used_mb=0)
            return self._json(200, {"ok": True, "released": True})

        if path == "/v1/switch_model":
            model = body.get("model")
            if not model:
                return self._json(400, {"ok": False, "error": "model_required"})
            STATE.update(active="llm", model=model, gpu_used_mb=6800)
            return self._json(200, {"status": "switched", "model": model, "service": "llm"})

        if path in ("/v1/chat/completions", "/v1/completions"):
            STATE["requests"] += 1
            return self._json(200, {
                "id": "chatcmpl-mock",
                "object": "chat.completion",
                "created": int(time.time()),
                "model": STATE["model"] or "mock",
                "choices": [{
                    "index": 0,
                    "message": {"role": "assistant", "content": "Risposta mock dal Node Agent. Nessun modello reale è stato caricato."},
                    "finish_reason": "stop",
                }],
                "usage": {"prompt_tokens": 10, "completion_tokens": 12, "total_tokens": 22},
                "provider_used": "mock-node-agent",
            })

        if path == "/v1/images/generations":
            return self._json(200, {
                "created": int(time.time()),
                "data": [{"url": "mock://image/0001.png", "revised_prompt": body.get("prompt", "")}],
            })

        if path == "/v1/audio/speech":
            audio = b"RIFF$\x00\x00\x00WAVEfmt " + b"\x00" * 32
            self.send_response(200)
            self.send_header("Content-Type", "audio/wav")
            self.send_header("Content-Length", str(len(audio)))
            self.end_headers()
            self.wfile.write(audio)
            return

        if path == "/v1/audio/transcriptions":
            return self._json(200, {"text": "Trascrizione mock.", "language": "it"})

        return self._json(404, {"ok": False, "error": "not_found"})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=18080)
    args = ap.parse_args()
    srv = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    print(f"[mock-node] in ascolto su http://127.0.0.1:{args.port}", flush=True)
    srv.serve_forever()


if __name__ == "__main__":
    main()
