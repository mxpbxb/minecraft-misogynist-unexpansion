// The Key Binds screen's search by key (Controlling's, by Jaredlll08, MIT): while that screen lays
// itself out, each binding's key, by the binding's name, for `lang::get` to add to the name its
// filter searches.
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The keys, and when the Key Binds screen last laid itself out with them.
static KEYS: Mutex<Option<(Instant, HashMap<String, String>)>> = Mutex::new(None);

/// It is laid out every frame it is open: past this, it has closed, and names are names again.
const FRESH: Duration = Duration::from_millis(250);

/// The Key Binds screen is laying itself out with these keys (binding name, then what to add), or
/// any other page is (None).
pub fn arm(keys: Option<HashMap<String, String>>) {
    if let Ok(mut k) = KEYS.lock() {
        *k = keys.map(|keys| (Instant::now(), keys));
    }
}

/// What to add to a binding's name, while the Key Binds screen lays itself out.
pub fn extra(name: &str) -> Option<String> {
    let k = KEYS.lock().ok()?;
    let (at, keys) = k.as_ref()?;
    if at.elapsed() > FRESH {
        return None;
    }
    keys.get(name).cloned()
}
