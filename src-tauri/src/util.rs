//! Small shared helpers.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a process-unique id with a human-readable prefix, e.g. `agent-1a2b-3`.
/// Avoids pulling in a uuid dependency; uniqueness comes from time + a counter.
pub fn new_id(prefix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefix}-{:x}-{}", nanos & 0xffff_ffff, n)
}
