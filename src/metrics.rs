//! Telemetria reale del gateway.
//! Registra richieste, errori, latenza per endpoint/intent/provider e uptime.
//! Sostituisce i valori finti che erano hardcoded in /stats.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct EndpointStats {
    pub requests: u64,
    pub errors: u64,
}

#[derive(Debug, Clone, Default)]
pub struct IntentStats {
    pub requests: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    pub requests: u64,
    pub errors: u64,
    pub last_latency_ms: u64,
    pub total_latency_ms: u64,
}

impl ProviderStats {
    pub fn avg_latency_ms(&self) -> u64 {
        if self.requests == 0 {
            0
        } else {
            self.total_latency_ms / self.requests
        }
    }
}

/// Ring buffer per la sparkline della latenza (ultimi N valori).
#[derive(Debug, Clone)]
pub struct LatencyRing {
    pub values: Vec<u64>,
    pub capacity: usize,
}

impl LatencyRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, v: u64) {
        if self.values.len() >= self.capacity {
            self.values.remove(0);
        }
        self.values.push(v);
    }

    pub fn avg(&self) -> u64 {
        if self.values.is_empty() {
            0
        } else {
            self.values.iter().sum::<u64>() / self.values.len() as u64
        }
    }

    pub fn max(&self) -> u64 {
        self.values.iter().copied().max().unwrap_or(0)
    }
}

/// Registry principale delle metriche, condiviso via Arc<Mutex<MetricsRegistry>>.
#[derive(Debug)]
pub struct MetricsRegistry {
    pub started_at: Instant,
    pub total_requests: AtomicU64,
    pub total_errors: AtomicU64,
    pub endpoints: HashMap<String, EndpointStats>,
    pub intents: HashMap<String, IntentStats>,
    pub providers: HashMap<String, ProviderStats>,
    pub latency_ring: LatencyRing,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            endpoints: HashMap::new(),
            intents: HashMap::new(),
            providers: HashMap::new(),
            latency_ring: LatencyRing::new(60),
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Registra l'inizio di una richiesta (endpoint + intent).
    pub fn record_start(&mut self, endpoint: &str, intent: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.endpoints
            .entry(endpoint.to_string())
            .or_default()
            .requests += 1;
        self.intents
            .entry(intent.to_string())
            .or_default()
            .requests += 1;
    }

    /// Registra l'esito di una richiesta verso un provider.
    pub fn record_provider(
        &mut self,
        provider: &str,
        ok: bool,
        latency_ms: u64,
    ) {
        if !ok {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }
        let p = self.providers.entry(provider.to_string()).or_default();
        p.requests += 1;
        if !ok {
            p.errors += 1;
        }
        p.last_latency_ms = latency_ms;
        p.total_latency_ms += latency_ms;
        self.latency_ring.push(latency_ms);
    }

    /// Registra un errore a livello di endpoint (es. nessun provider disponibile).
    pub fn record_endpoint_error(&mut self, endpoint: &str) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
        self.endpoints
            .entry(endpoint.to_string())
            .or_default()
            .errors += 1;
    }

    pub fn snapshot(&self) -> serde_json::Value {
        let uptime = self.uptime_secs();
        let uptime_h = uptime / 3600;
        let uptime_m = (uptime % 3600) / 60;
        let uptime_s = uptime % 60;
        let uptime_human = if uptime_h > 0 {
            format!("{}h {}m {}s", uptime_h, uptime_m, uptime_s)
        } else if uptime_m > 0 {
            format!("{}m {}s", uptime_m, uptime_s)
        } else {
            format!("{}s", uptime_s)
        };

        let mut endpoints = serde_json::Map::new();
        for (k, v) in &self.endpoints {
            endpoints.insert(
                k.clone(),
                serde_json::json!({
                    "requests": v.requests,
                    "errors": v.errors,
                    "error_rate": if v.requests > 0 {
                        (v.errors as f64 / v.requests as f64 * 100.0).round()
                    } else { 0.0 },
                }),
            );
        }

        let mut intents = serde_json::Map::new();
        for (k, v) in &self.intents {
            intents.insert(k.clone(), serde_json::json!({ "requests": v.requests }));
        }

        let mut providers = serde_json::Map::new();
        for (k, v) in &self.providers {
            providers.insert(
                k.clone(),
                serde_json::json!({
                    "requests": v.requests,
                    "errors": v.errors,
                    "error_rate": if v.requests > 0 {
                        (v.errors as f64 / v.requests as f64 * 100.0).round()
                    } else { 0.0 },
                    "last_latency_ms": v.last_latency_ms,
                    "avg_latency_ms": v.avg_latency_ms(),
                }),
            );
        }

        let total = self.total_requests.load(Ordering::Relaxed);
        let errs = self.total_errors.load(Ordering::Relaxed);
        let error_rate = if total > 0 {
            (errs as f64 / total as f64 * 100.0).round()
        } else {
            0.0
        };

        serde_json::json!({
            "uptime_secs": uptime,
            "uptime_human": uptime_human,
            "total_requests": total,
            "total_errors": errs,
            "error_rate": error_rate,
            "last_latency_ms": self.latency_ring.values.last().copied().unwrap_or(0),
            "avg_latency_ms": self.latency_ring.avg(),
            "max_latency_ms": self.latency_ring.max(),
            "latency_ring": self.latency_ring.values,
            "endpoints": endpoints,
            "intents": intents,
            "providers": providers,
        })
    }
}
