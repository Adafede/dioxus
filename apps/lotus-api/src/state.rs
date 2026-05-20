// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::{
    config::AppConfig,
    types::{ExportUrlResponse, HealthResponse, SearchResponse},
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{OnceCell, Semaphore};

pub(crate) const TAXON_CACHE_TTL: Duration = Duration::from_secs(60 * 60 * 24);
pub(crate) const SEARCH_CACHE_TTL: Duration = Duration::from_secs(60 * 3);
pub(crate) const EXPORT_CACHE_TTL: Duration = Duration::from_secs(60 * 10);
const CACHE_PRUNE_INTERVAL: Duration = Duration::from_secs(20);
const MAX_TAXON_CACHE_ENTRIES: usize = 512;
const MAX_SEARCH_CACHE_ENTRIES: usize = 128;
const MAX_EXPORT_CACHE_ENTRIES: usize = 256;

pub(crate) type InFlightSearch =
    Arc<OnceCell<Result<SearchResponse, crate::errors::SharedApiError>>>;
pub(crate) type InFlightExport =
    Arc<OnceCell<Result<ExportUrlResponse, crate::errors::SharedApiError>>>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) default_limit: usize,
    pub(crate) request_timeout: Duration,
    pub(crate) request_permits: Arc<Semaphore>,
    pub(crate) taxon_cache: Arc<Mutex<HashMap<String, CachedTaxonResolution>>>,
    pub(crate) search_cache: Arc<Mutex<HashMap<String, CachedSearchResponse>>>,
    pub(crate) export_cache: Arc<Mutex<HashMap<String, CachedExportResponse>>>,
    pub(crate) search_inflight: Arc<Mutex<HashMap<String, InFlightSearch>>>,
    pub(crate) export_inflight: Arc<Mutex<HashMap<String, InFlightExport>>>,
    pub(crate) taxon_cache_prune_after: Arc<Mutex<Instant>>,
    pub(crate) search_cache_prune_after: Arc<Mutex<Instant>>,
    pub(crate) export_cache_prune_after: Arc<Mutex<Instant>>,
    pub(crate) metrics: Arc<RuntimeMetrics>,
}

impl AppState {
    pub(crate) fn new(config: &AppConfig) -> Self {
        Self {
            default_limit: config.default_limit,
            request_timeout: config.request_timeout,
            request_permits: Arc::new(Semaphore::new(config.max_concurrency)),
            taxon_cache: Arc::new(Mutex::new(HashMap::new())),
            search_cache: Arc::new(Mutex::new(HashMap::new())),
            export_cache: Arc::new(Mutex::new(HashMap::new())),
            search_inflight: Arc::new(Mutex::new(HashMap::new())),
            export_inflight: Arc::new(Mutex::new(HashMap::new())),
            taxon_cache_prune_after: Arc::new(Mutex::new(Instant::now() + CACHE_PRUNE_INTERVAL)),
            search_cache_prune_after: Arc::new(Mutex::new(Instant::now() + CACHE_PRUNE_INTERVAL)),
            export_cache_prune_after: Arc::new(Mutex::new(Instant::now() + CACHE_PRUNE_INTERVAL)),
            metrics: Arc::new(RuntimeMetrics::new()),
        }
    }
}

#[derive(Clone)]
pub(crate) struct CachedTaxonResolution {
    pub(crate) inserted_at: Instant,
    pub(crate) value: (Option<String>, Option<String>),
}

#[derive(Clone)]
pub(crate) struct CachedSearchResponse {
    pub(crate) inserted_at: Instant,
    pub(crate) value: SearchResponse,
}

#[derive(Clone)]
pub(crate) struct CachedExportResponse {
    pub(crate) inserted_at: Instant,
    pub(crate) value: ExportUrlResponse,
}

pub(crate) struct RuntimeMetrics {
    started_at: Instant,
    pub(crate) search_cache_hits: AtomicU64,
    pub(crate) search_cache_misses: AtomicU64,
    pub(crate) search_inflight_waits: AtomicU64,
    pub(crate) search_upstream_hits: AtomicU64,
    pub(crate) export_cache_hits: AtomicU64,
    pub(crate) export_cache_misses: AtomicU64,
    pub(crate) export_inflight_waits: AtomicU64,
    pub(crate) export_upstream_hits: AtomicU64,
    pub(crate) overload_rejections: AtomicU64,
    pub(crate) request_timeouts: AtomicU64,
}

impl RuntimeMetrics {
    pub(crate) fn new() -> Self {
        Self {
            started_at: Instant::now(),
            search_cache_hits: AtomicU64::new(0),
            search_cache_misses: AtomicU64::new(0),
            search_inflight_waits: AtomicU64::new(0),
            search_upstream_hits: AtomicU64::new(0),
            export_cache_hits: AtomicU64::new(0),
            export_cache_misses: AtomicU64::new(0),
            export_inflight_waits: AtomicU64::new(0),
            export_upstream_hits: AtomicU64::new(0),
            overload_rejections: AtomicU64::new(0),
            request_timeouts: AtomicU64::new(0),
        }
    }

    pub(crate) fn snapshot(&self) -> HealthResponse {
        HealthResponse {
            status: "ok",
            uptime_secs: self.started_at.elapsed().as_secs(),
            search_cache_hits: self.search_cache_hits.load(Ordering::Relaxed),
            search_cache_misses: self.search_cache_misses.load(Ordering::Relaxed),
            search_inflight_waits: self.search_inflight_waits.load(Ordering::Relaxed),
            search_upstream_hits: self.search_upstream_hits.load(Ordering::Relaxed),
            export_cache_hits: self.export_cache_hits.load(Ordering::Relaxed),
            export_cache_misses: self.export_cache_misses.load(Ordering::Relaxed),
            export_inflight_waits: self.export_inflight_waits.load(Ordering::Relaxed),
            export_upstream_hits: self.export_upstream_hits.load(Ordering::Relaxed),
            overload_rejections: self.overload_rejections.load(Ordering::Relaxed),
            request_timeouts: self.request_timeouts.load(Ordering::Relaxed),
        }
    }
}

pub(crate) fn build_search_cache_key(query: &str, limit: usize, include_counts: bool) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"search");
    hasher.update(limit.to_le_bytes());
    hasher.update([include_counts as u8]);
    hasher.update(query.as_bytes());
    format!("search:{}", sha256_hex(hasher.finalize()))
}

pub(crate) fn build_export_cache_key(query: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"export");
    hasher.update(query.as_bytes());
    format!("export:{}", sha256_hex(hasher.finalize()))
}

fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

pub(crate) fn search_cache_get(state: &AppState, key: &str) -> Option<SearchResponse> {
    let mut cache = state.search_cache.lock().ok()?;
    maybe_prune_cache(
        &mut cache,
        &state.search_cache_prune_after,
        SEARCH_CACHE_TTL,
        MAX_SEARCH_CACHE_ENTRIES,
        |entry| entry.inserted_at,
    );
    cache.get(key).map(|entry| entry.value.clone())
}

pub(crate) fn search_cache_put(state: &AppState, key: String, value: SearchResponse) {
    if let Ok(mut cache) = state.search_cache.lock() {
        maybe_prune_cache(
            &mut cache,
            &state.search_cache_prune_after,
            SEARCH_CACHE_TTL,
            MAX_SEARCH_CACHE_ENTRIES,
            |entry| entry.inserted_at,
        );
        cache.insert(
            key,
            CachedSearchResponse {
                inserted_at: Instant::now(),
                value,
            },
        );
    }
}

pub(crate) fn export_cache_get(state: &AppState, key: &str) -> Option<ExportUrlResponse> {
    let mut cache = state.export_cache.lock().ok()?;
    maybe_prune_cache(
        &mut cache,
        &state.export_cache_prune_after,
        EXPORT_CACHE_TTL,
        MAX_EXPORT_CACHE_ENTRIES,
        |entry| entry.inserted_at,
    );
    cache.get(key).map(|entry| entry.value.clone())
}

pub(crate) fn export_cache_put(state: &AppState, key: String, value: ExportUrlResponse) {
    if let Ok(mut cache) = state.export_cache.lock() {
        maybe_prune_cache(
            &mut cache,
            &state.export_cache_prune_after,
            EXPORT_CACHE_TTL,
            MAX_EXPORT_CACHE_ENTRIES,
            |entry| entry.inserted_at,
        );
        cache.insert(
            key,
            CachedExportResponse {
                inserted_at: Instant::now(),
                value,
            },
        );
    }
}

pub(crate) fn search_inflight_cell(state: &AppState, key: &str) -> (InFlightSearch, bool) {
    let mut inflight = state.search_inflight.lock().expect("search inflight mutex");
    if let Some(existing) = inflight.get(key) {
        return (existing.clone(), false);
    }
    let cell = Arc::new(OnceCell::new());
    inflight.insert(key.to_string(), cell.clone());
    (cell, true)
}

pub(crate) fn search_inflight_remove(
    state: &AppState,
    key: &str,
    cell: &InFlightSearch,
    is_leader: bool,
) {
    if !is_leader {
        return;
    }
    if let Ok(mut inflight) = state.search_inflight.lock()
        && inflight
            .get(key)
            .is_some_and(|current| Arc::ptr_eq(current, cell))
    {
        inflight.remove(key);
    }
}

pub(crate) fn export_inflight_cell(state: &AppState, key: &str) -> (InFlightExport, bool) {
    let mut inflight = state.export_inflight.lock().expect("export inflight mutex");
    if let Some(existing) = inflight.get(key) {
        return (existing.clone(), false);
    }
    let cell = Arc::new(OnceCell::new());
    inflight.insert(key.to_string(), cell.clone());
    (cell, true)
}

pub(crate) fn export_inflight_remove(
    state: &AppState,
    key: &str,
    cell: &InFlightExport,
    is_leader: bool,
) {
    if !is_leader {
        return;
    }
    if let Ok(mut inflight) = state.export_inflight.lock()
        && inflight
            .get(key)
            .is_some_and(|current| Arc::ptr_eq(current, cell))
    {
        inflight.remove(key);
    }
}

pub(crate) fn taxon_cache_get(
    state: &AppState,
    key: &str,
) -> Option<(Option<String>, Option<String>)> {
    let mut cache = state.taxon_cache.lock().ok()?;
    maybe_prune_cache(
        &mut cache,
        &state.taxon_cache_prune_after,
        TAXON_CACHE_TTL,
        MAX_TAXON_CACHE_ENTRIES,
        |entry| entry.inserted_at,
    );
    cache.get(key).map(|entry| entry.value.clone())
}

pub(crate) fn taxon_cache_put(
    state: &AppState,
    key: String,
    value: (Option<String>, Option<String>),
) {
    if let Ok(mut cache) = state.taxon_cache.lock() {
        maybe_prune_cache(
            &mut cache,
            &state.taxon_cache_prune_after,
            TAXON_CACHE_TTL,
            MAX_TAXON_CACHE_ENTRIES,
            |entry| entry.inserted_at,
        );
        cache.insert(
            key,
            CachedTaxonResolution {
                inserted_at: Instant::now(),
                value,
            },
        );
    }
}

fn maybe_prune_cache<V, F>(
    cache: &mut HashMap<String, V>,
    prune_after: &Arc<Mutex<Instant>>,
    ttl: Duration,
    max_entries: usize,
    inserted_at: F,
) where
    F: Fn(&V) -> Instant,
{
    let now = Instant::now();
    let should_prune = if let Ok(mut next_prune) = prune_after.lock() {
        if now < *next_prune {
            false
        } else {
            *next_prune = now + CACHE_PRUNE_INTERVAL;
            true
        }
    } else {
        true
    };

    if should_prune {
        prune_cache(cache, ttl, max_entries, inserted_at);
    }
}

pub(crate) fn prune_cache<V, F>(
    cache: &mut HashMap<String, V>,
    ttl: Duration,
    max_entries: usize,
    inserted_at: F,
) where
    F: Fn(&V) -> Instant,
{
    let now = Instant::now();
    cache.retain(|_, value| now.duration_since(inserted_at(value)) <= ttl);
    while cache.len() > max_entries {
        let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, value)| inserted_at(value))
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        cache.remove(&oldest_key);
    }
}
