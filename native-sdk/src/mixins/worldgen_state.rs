// Misogynist Unexpansion's world generation settings, from `worldgen.cfg` in the mod's config
// folder: how much of the deep underground is deep dark, and whether the deep dark keeps its
// sculk. Written with the defaults when missing; read again at most every ten seconds.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub(crate) const FILE: &str = "worldgen.cfg";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settings {
    /// 1.0 is vanilla's deep dark; 2.0 about twice as much, 0.5 about half, 0 none.
    pub(crate) deep_dark: f64,
    /// Whether the deep dark's sculk patches (sculk, sensors, shriekers) and sculk veins generate.
    pub(crate) sculk: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { deep_dark: 1.0, sculk: true }
    }
}

pub(crate) fn text(s: Settings) -> String {
    format!(
        "# Minecraft Misogynist Unexpansion: the deep dark, in new terrain.\n\
         # deep_dark: how much of the deep underground is deep dark. 1.0 is vanilla's; higher is\n\
         # more (2.0 about twice as much), lower is less (0.5 about half, 0 none).\n\
         # sculk: on, the deep dark has its sculk, sensors and shriekers (and so its warden);\n\
         # off, it is bare deepslate caves and sculk is only crafted.\n\
         # Takes hold for terrain generated after the world is next opened.\n\
         # Delete this file to get the defaults back.\n\n\
         deep_dark = {}\n\
         sculk = {}\n",
        number(s.deep_dark),
        if s.sculk { "on" } else { "off" },
    )
}

fn number(v: f64) -> String {
    if v.fract() == 0.0 { format!("{v:.1}") } else { format!("{v}") }
}

/// Reads a file in `text`'s form: a line it doesn't understand keeps its default.
pub(crate) fn parse(text: &str) -> Settings {
    let mut s = Settings::default();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim().to_ascii_lowercase();
        match key.trim() {
            "deep_dark" => {
                if let Ok(v) = value.parse::<f64>()
                    && v.is_finite()
                    && v >= 0.0
                {
                    s.deep_dark = v;
                }
            }
            "sculk" => match value.as_str() {
                "on" | "true" | "yes" | "kept" => s.sculk = true,
                "off" | "false" | "no" | "removed" => s.sculk = false,
                _ => {}
            },
            _ => {}
        }
    }
    s
}

/// The settings now: the file's, read again when ten seconds have passed since the last read.
pub(crate) fn settings() -> Settings {
    static CACHE: Mutex<Option<(Instant, Settings)>> = Mutex::new(None);
    let Ok(mut cache) = CACHE.lock() else { return Settings::default() };
    if let Some((at, s)) = *cache
        && at.elapsed() < Duration::from_secs(10)
    {
        return s;
    }
    let s = read();
    *cache = Some((Instant::now(), s));
    s
}

fn read() -> Settings {
    if cfg!(test) {
        return Settings::default();
    }
    let Ok(dir) = std::env::current_dir() else { return Settings::default() };
    let dir = dir.join("config").join("misogynist_unexpansion_native");
    let path = dir.join(FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let s = Settings::default();
            let _ = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, text(s)));
            s
        }
        Err(_) => Settings::default(),
    }
}

thread_local! {
    static INSIDE: Cell<bool> = const { Cell::new(false) };
}

/// Whether this thread is already assembling the biome tables for the deep dark's hook.
pub(crate) fn inside() -> bool {
    INSIDE.with(Cell::get)
}

/// Runs `f` marked as inside the hook, so the hook it calls again lets the original run.
pub(crate) fn reentered<T>(f: impl FnOnce() -> T) -> T {
    INSIDE.with(|i| i.set(true));
    let out = f();
    INSIDE.with(|i| i.set(false));
    out
}

const EROSION: usize = 3;
/// A climate value as the tables hold it: vanilla's `Climate.quantizeCoord`, times 10000.
const TOP: i64 = 10_000;

/// Every multi-noise table's deep dark, vanilla's and each mod region's copy, with its erosion
/// range (vanilla's -1 to -0.375) made `factor` times as wide from its low end, up to the whole
/// range; at 0 the deep dark is taken out. A table it can't read is left as it is.
pub(crate) fn scale_deep_dark(presets: &mut BTreeMap<String, String>, factor: f64) {
    for text in presets.values_mut() {
        if !text.contains("\"minecraft:deep_dark\"") {
            continue;
        }
        let Ok(serde_json::Value::Array(mut table)) = serde_json::from_str::<serde_json::Value>(text) else {
            continue;
        };
        let deep = |e: &serde_json::Value| e["biome"].as_str() == Some("minecraft:deep_dark");
        if factor <= 0.0 {
            table.retain(|e| !deep(e));
        } else {
            for entry in table.iter_mut().filter(|e| deep(e)) {
                let Some(range) = entry["coordinates"].get_mut(EROSION).and_then(|r| r.as_array_mut()) else {
                    continue;
                };
                let (Some(low), Some(high)) = (range[0].as_i64(), range.get(1).and_then(|v| v.as_i64())) else {
                    continue;
                };
                let wide = ((high - low) as f64 * factor).round() as i64;
                range[1] = serde_json::Value::from((low + wide).clamp(low, TOP));
            }
        }
        if let Ok(new) = serde_json::to_string(&table) {
            *text = new;
        }
    }
}
