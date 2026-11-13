//! `removed_mobs.cfg` in the mod's config folder: which of the listed mobs are taken out of the game.
//! Only these mobs are listed, each `removed` or `kept`; everything else is always kept. Written with
//! the defaults (all removed) when missing, and read again when it changes, within ten seconds.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// The mobs this mod can take out, all of them taken out by default.
pub const MOBS: [&str; 32] = [
    "dolphin", "fox", "sniffer", "copper_golem", "allay", "turtle", "bee", "parrot", "husk", "stray",
    "polar_bear", "panda", "camel", "llama", "phantom", "frog", "glow_squid", "strider", "camel_husk",
    "armadillo", "axolotl", "happy_ghast", "sulfur_cube", "nautilus", "trader_llama", "zombie_nautilus",
    "bogged", "parched", "breeze", "creaking", "tadpole", "goat",
];

pub const FILE: &str = "removed_mobs.cfg";

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Each listed mob (no namespace), and whether it is removed.
    pub removed: HashMap<&'static str, bool>,
}

impl Default for Config {
    fn default() -> Config {
        Config { removed: MOBS.iter().map(|&m| (m, true)).collect() }
    }
}

impl Config {
    /// Whether the mob type `kind` (`minecraft:fox`) is taken out.
    pub fn removes(&self, kind: &str) -> bool {
        kind.strip_prefix("minecraft:").and_then(|k| self.removed.get(k)).copied().unwrap_or(false)
    }

    pub fn text(&self) -> String {
        let mut out = String::from(
            "# Minecraft Misogynist Unexpansion: the mobs it takes out of the game.\n\
             # removed: none spawn, and those already in a world are gone when they load.\n\
             # kept: back as in vanilla. Saved changes take hold within ten seconds.\n\
             # Delete this file to get the defaults back.\n\n",
        );
        for m in MOBS {
            let removed = self.removed.get(m).copied().unwrap_or(true);
            out += &format!("{m} = {}\n", if removed { "removed" } else { "kept" });
        }
        out
    }

    /// Reads a file in `text`'s form: a line it doesn't understand keeps its mob's default.
    pub fn parse(text: &str) -> Config {
        let mut c = Config::default();
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            let Some((key, value)) = line.split_once('=') else { continue };
            let Some(slot) = c.removed.get_mut(key.trim()) else { continue };
            match value.trim().to_ascii_lowercase().as_str() {
                "removed" | "remove" | "on" | "true" | "yes" => *slot = true,
                "kept" | "keep" | "off" | "false" | "no" => *slot = false,
                _ => {}
            }
        }
        c
    }
}

/// The config and its file, with when it was last read.
pub struct Loaded {
    pub config: Config,
    file: Option<(PathBuf, Option<SystemTime>)>,
    ticks: u32,
}

impl Loaded {
    pub fn new() -> Loaded {
        Loaded { config: Config::default(), file: None, ticks: 0 }
    }

    /// Every 200 calls: writes the defaults if the file is missing, and reads it again if it changed.
    pub fn refresh(&mut self, dir: impl FnOnce() -> Option<PathBuf>) {
        self.ticks += 1;
        if self.file.is_some() && self.ticks % 200 != 0 {
            return;
        }
        let path = match &self.file {
            Some((p, _)) => p.clone(),
            None => match dir() {
                Some(d) => d.join(FILE),
                None => return,
            },
        };
        if !path.exists() {
            let _ = std::fs::write(&path, Config::default().text());
        }
        let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if self.file.as_ref().is_some_and(|(_, t)| *t == modified) {
            return;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            self.config = Config::parse(&text);
        }
        self.file = Some((path, modified));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_remove_every_listed_mob_and_read_back() {
        let c = Config::default();
        assert!(MOBS.iter().all(|m| c.removes(&format!("minecraft:{m}"))));
        assert_eq!(Config::parse(&c.text()), c);
    }

    #[test]
    fn only_the_listed_mobs_can_be_removed() {
        let c = Config::parse("zombie = removed\ncreeper = removed\nfox = kept\n");
        assert!(!c.removes("minecraft:zombie"));
        assert!(!c.removes("minecraft:creeper"));
        assert!(!c.removes("minecraft:fox"));
        assert!(c.removes("minecraft:goat"));
        assert!(!c.text().contains("\nzombie ="));
        assert!(!c.text().contains("\ncreeper ="));
    }
}
