//! Minecraft Misogynist Unexpansion: the game as it was, more or less.
//!
//! - The mobs in `removed_mobs.cfg` (dolphins, foxes, bees, goats and the rest of the newer ones by
//!   default) never join a world: none spawn, and those already in one are gone when they load.
//! - Cows, pigs and chickens come only in their original (temperate) look, old ones included.
//! - Babies look as they did before the baby redesign: villagers, zombies, piglins, hoglins, farm
//!   animals, wolves, cats, rabbits, horses and squid drawn with the grown model and textures at the
//!   old sizes (`babies.rs`).
//! - Tropical fish drop fish scales (turtle scutes), nine times in ten; turtle scutes and eggs are
//!   called fish scales and fish eggs.
//! - Fishing as before the open-water rule: treasure anywhere, a pond as an ocean; and tropical fish
//!   five in a hundred more often (7, out of cod's share).
//!
//! The recipes and the Potion of Dolphin's Grace are the content mod `misogynist_unexpansion`.

mod babies;
mod config;

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use horizon_client::mod_api::{
    EntityJoinLevelEvent, Flow, Hooks, Host, ItemStack, LevelTickPostEvent, LootDropsEvent, LootTableLoadEvent,
    MobSubmitEvent, Tag, Text, TooltipEvent, serde_json::Value,
};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod)
}

struct Mod;

static CONFIG: LazyLock<Mutex<config::Loaded>> = LazyLock::new(|| Mutex::new(config::Loaded::new()));
/// Cows, pigs and chickens that just joined a world, to set to the temperate variant next tick.
static TO_REVERT: Mutex<Vec<u32>> = Mutex::new(Vec::new());
/// Ticks between reads of the babies' looks.
static SCAN: AtomicU32 = AtomicU32::new(0);
static ROLL: AtomicU64 = AtomicU64::new(0x9E37_79B9_7F4A_7C15);

/// A number from 0 to 99.
fn percent() -> u64 {
    let mut x = ROLL.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    ROLL.store(x, Ordering::Relaxed);
    x % 100
}

/// The farm animals that have variants: only the temperate one is kept.
fn has_variant(kind: &str) -> bool {
    matches!(kind, "minecraft:cow" | "minecraft:pig" | "minecraft:chicken")
}

/// How often tropical fish drop fish scales, in 100.
const SCALES_CHANCE: u64 = 90;

/// Takes out every loot condition that asks for open water (`FishingHook.isOpenWaterFishing`):
/// treasure is caught anywhere again, as before 1.16.
fn drop_open_water(v: &mut Value) -> usize {
    let open_water = |c: &Value| c.to_string().contains("in_open_water");
    let mut dropped = 0;
    match v {
        Value::Object(map) => {
            if map.get("condition").is_some_and(open_water) {
                map.remove("condition");
                dropped += 1;
            }
            if let Some(Value::Array(list)) = map.get_mut("conditions") {
                let before = list.len();
                list.retain(|c| !open_water(c));
                dropped += before - list.len();
            }
            for child in map.values_mut() {
                dropped += drop_open_water(child);
            }
        }
        Value::Array(list) => {
            for child in list {
                dropped += drop_open_water(child);
            }
        }
        _ => {}
    }
    dropped
}

/// Tropical fish five in a hundred more often, out of cod's share: 7 and 55, of 100.
fn more_tropical_fish(v: &mut Value) {
    let Some(pools) = v.get_mut("pools").and_then(Value::as_array_mut) else { return };
    for pool in pools {
        let Some(entries) = pool.get_mut("entries").and_then(Value::as_array_mut) else { continue };
        for e in entries {
            let weight = match e.get("name").and_then(Value::as_str) {
                Some("minecraft:tropical_fish") => 7,
                Some("minecraft:cod") => 55,
                _ => continue,
            };
            e["weight"] = Value::from(weight);
        }
    }
}

impl Hooks for Mod {
    /// Fishing as it was: no open-water rule, and more tropical fish.
    fn loot_table_load(&self, event: &mut LootTableLoadEvent<'_>) {
        match event.id {
            "minecraft:gameplay/fishing" => {
                drop_open_water(event.table);
            }
            "minecraft:gameplay/fishing/fish" => more_tropical_fish(event.table),
            _ => {}
        }
    }

    /// A removed mob never joins: not spawned, not loaded. A cow, pig or chicken that joins is set
    /// to its temperate variant on the next tick.
    fn entity_join_level(&self, host: &mut dyn Host, event: &mut EntityJoinLevelEvent<'_>) -> Flow {
        let removes = CONFIG
            .lock()
            .map(|mut c| {
                c.refresh(|| host.config_dir().ok());
                c.config.removes(event.kind)
            })
            .unwrap_or(false);
        if removes {
            return Flow::Cancel;
        }
        if has_variant(event.kind)
            && let Ok(mut v) = TO_REVERT.lock()
        {
            v.push(event.entity);
        }
        Flow::Continue
    }

    fn level_tick_post(&self, host: &mut dyn Host, event: &mut LevelTickPostEvent<'_>) {
        if SCAN.fetch_add(1, Ordering::Relaxed) % 10 == 0 {
            babies::scan(host, event.dimension);
        }
        if let Ok(mut c) = CONFIG.lock() {
            c.refresh(|| host.config_dir().ok());
        }
        let ids: Vec<u32> = TO_REVERT.lock().map(|mut v| std::mem::take(&mut *v)).unwrap_or_default();
        for id in ids {
            if let Some(entity) = host.entity(id) {
                host.entity_merge_nbt(entity, [("variant".into(), Tag::String("minecraft:temperate".into()))].into());
            }
        }
    }

    /// Fish scales from tropical fish.
    fn loot_drops(&self, _: &mut dyn Host, event: &mut LootDropsEvent<'_>) {
        if let Some((_, "minecraft:tropical_fish")) = event.context.entity
            && percent() < SCALES_CHANCE
        {
            event.drops.push(ItemStack::new("minecraft:turtle_scute", 1));
        }
    }

    /// Turtles are gone: their scutes are fish scales and their eggs fish eggs. The name is the
    /// tooltip's first line; a stack renamed in an anvil keeps its own.
    fn tooltip(&self, _: &dyn Host, event: &mut TooltipEvent<'_>) {
        let name = match &*event.stack.item {
            "minecraft:turtle_scute" => "Fish Scales",
            "minecraft:turtle_egg" => "Fish Eggs",
            _ => return,
        };
        let renamed = event.stack.extra.as_ref().is_some_and(|d| d.custom_name.is_some());
        if !renamed && let Some(first) = event.lines.first_mut() {
            first.0 = Text::literal(name);
        }
    }

    fn mob_submit(&self, host: &dyn Host, event: &mut MobSubmitEvent<'_>) -> Flow {
        babies::draw(host, event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vanilla's tables as the game loads them.
    const FISHING: &str = r#"{"type": "minecraft:fishing", "pools": [{"entries": [{"type": "minecraft:loot_table", "quality": -2, "value": "minecraft:gameplay/fishing/junk", "weight": 10}, {"type": "minecraft:loot_table", "condition": {"type": "minecraft:entity_properties", "entity": "this", "predicate": {"minecraft:type_specific/fishing_hook": {"in_open_water": true}}}, "quality": 2, "value": "minecraft:gameplay/fishing/treasure", "weight": 5}, {"type": "minecraft:loot_table", "quality": -1, "value": "minecraft:gameplay/fishing/fish", "weight": 85}], "rolls": 1}]}"#;
    const FISH: &str = r#"{"pools": [{"entries": [{"type": "minecraft:item", "name": "minecraft:cod", "weight": 60}, {"type": "minecraft:item", "name": "minecraft:salmon", "weight": 25}, {"type": "minecraft:item", "name": "minecraft:tropical_fish", "weight": 2}, {"type": "minecraft:item", "name": "minecraft:pufferfish", "weight": 13}], "rolls": 1}]}"#;

    #[test]
    fn treasure_no_longer_needs_open_water() {
        let mut t: Value = horizon_client::mod_api::serde_json::from_str(FISHING).unwrap();
        assert_eq!(drop_open_water(&mut t), 1);
        assert!(!t.to_string().contains("in_open_water"));
        // The treasure entry itself stays, at its weight.
        assert!(t.to_string().contains("gameplay/fishing/treasure"));
    }

    #[test]
    fn tropical_fish_come_five_in_a_hundred_more_often() {
        let mut t: Value = horizon_client::mod_api::serde_json::from_str(FISH).unwrap();
        more_tropical_fish(&mut t);
        let entries = t["pools"][0]["entries"].as_array().unwrap();
        let w = |n: &str| entries.iter().find(|e| e["name"] == n).unwrap()["weight"].as_i64().unwrap();
        assert_eq!((w("minecraft:tropical_fish"), w("minecraft:cod")), (7, 55));
        assert_eq!(entries.iter().map(|e| e["weight"].as_i64().unwrap()).sum::<i64>(), 100);
    }
}
