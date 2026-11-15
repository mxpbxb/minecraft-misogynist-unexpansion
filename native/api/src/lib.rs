//! Minecraft Misogynist Unexpansion: the game as it was, more or less.
//!
//! - The mobs in `removed_mobs.cfg` (dolphins, foxes, bees, goats and the rest of the newer ones by
//!   default) never join a world: none spawn, and those already in one are gone when they load.
//! - No new looks for old mobs: cows, pigs and chickens only temperate, wolves only the original
//!   (pale) wolf, old ones included (checked every two seconds; the mixin `original-variants` draws
//!   them so, even on a server without this mod).
//! - Kelp only in warm, lukewarm and frozen oceans, in new terrain (the mixin `kelp-biomes`).
//! - Animals make their classic sounds: no sound variants, and babies sound as they did (the mixin
//!   `classic-sounds`).
//! - Enchanting costs half the level it asks for: a level 30 enchantment takes 15 levels
//!   (`enchant.rs`, and the mixin `half-level-enchant` in single player).
//! - Powder snow is walked on, leather boots or not (the mixin `powder-snow-walk`).
//! - Babies look as they did before the baby redesign: villagers, zombies, piglins, hoglins, farm
//!   animals, wolves, cats, rabbits, horses and squid drawn with the grown model and textures at the
//!   old sizes (`babies.rs`).
//! - Tropical fish drop fish scales (turtle scutes), nine times in ten; turtle scutes and eggs are
//!   called fish scales and fish eggs.
//! - Fishing as before the open-water rule: treasure anywhere, a pond as an ocean; and tropical fish
//!   five in a hundred more often (7, out of cod's share).
//! - No sculk patches or sculk veins in new terrain (the mixins `no-sculk-placed` and
//!   `no-sculk-feature`); sculk is crafted instead, and a bottle o' enchanting from mob heads
//!   (`farming.rs`).
//! - Right-clicking a grown crop harvests it and replants it (`farming.rs`).
//! - Netherite in an anvil: a diamond item and a netherite ingot, no smithing template (`anvil.rs`).
//! - Deepslate digs only 15% slower than stone (cobbled deepslate than cobblestone, its ores than
//!   theirs), not twice as slow.
//! - No leaf litter, wildflowers or pink petals on the ground: none in new terrain (the same
//!   mixins), and those already there gone as each chunk loads, once; they are smelted instead,
//!   from leaves, flowers and pink tulips (`ground.rs`).
//!
//! The first recipes and the Potion of Dolphin's Grace are the content mod `misogynist_unexpansion`;
//! the sculk ones come from this native half, so that content file stays as servers know it.

mod anvil;
mod babies;
mod config;
mod enchant;
mod farming;
mod ground;

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use horizon_client::mod_api::{
    AnvilUpdateEvent, BreakSpeedEvent, ChunkLoadEvent, DataPackFilesEvent, EnchantItemEvent, EntityJoinLevelEvent, Flow, Hooks, Host, ItemStack, LevelTickPostEvent, LootDropsEvent,
    LootTableLoadEvent, MobSubmitEvent, NetFromServerEvent, PrepareItemEnchantEvent, RightClickBlockEvent, Tag, Text, TooltipEvent, serde_json::Value,
};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod)
}

struct Mod;

static CONFIG: LazyLock<Mutex<config::Loaded>> = LazyLock::new(|| Mutex::new(config::Loaded::new()));
/// Cows, pigs and chickens that just joined a world, to set to the temperate variant next tick.
static TO_REVERT: Mutex<Vec<(u32, String)>> = Mutex::new(Vec::new());
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

/// The mobs that have newer looks: only the original one is kept.
fn has_variant(kind: &str) -> bool {
    original_look(kind).is_some()
}

/// The one look a mob keeps, as (key, value) pairs of its saved data: the newer variants are gone.
fn original_look(kind: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match kind {
        "minecraft:cow" | "minecraft:pig" | "minecraft:chicken" => Some(&[("variant", "minecraft:temperate")]),
        "minecraft:wolf" => Some(&[("variant", "minecraft:pale"), ("sound_variant", "minecraft:classic")]),
        _ => None,
    }
}

/// A mob with a newer look set to its original one. A key it doesn't save (yet) is left alone.
fn revert_look(host: &mut dyn Host, id: u32, kind: &str) {
    let Some(look) = original_look(kind) else { return };
    let Some(entity) = host.entity(id) else { return };
    let Some(tag) = host.entity_nbt(entity) else { return };
    let mut change = horizon_client::mod_api::Compound::default();
    for &(key, want) in look {
        let now = match tag.get(&key.into()) {
            Some(Tag::String(v)) => String::from_utf16_lossy(&v.0),
            _ => continue,
        };
        if now != want {
            change.insert(key.into(), Tag::String(want.into()));
        }
    }
    if !change.is_empty() {
        host.entity_merge_nbt(entity, change);
    }
}

/// How much faster deepslate digs: its hardness made 15% more than its stone counterpart's
/// (vanilla: deepslate 3, stone 1.5; cobbled deepslate 3.5, cobblestone 2; deepslate ores 4.5,
/// ores 3; polished deepslate, its bricks and tiles 3.5, stone bricks 1.5). None for anything else.
fn deepslate_speedup(block: &str) -> Option<f32> {
    let name = block.strip_prefix("minecraft:")?;
    if !name.contains("deepslate") || name.starts_with("infested") || name.starts_with("reinforced") {
        return None;
    }
    let (hardness, stone) = if name == "deepslate" {
        (3.0, 1.5)
    } else if name.ends_with("_ore") {
        (4.5, 3.0)
    } else if name.starts_with("cobbled_deepslate") {
        (3.5, 2.0)
    } else {
        (3.5, 1.5)
    };
    Some(hardness / (stone * 1.15))
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
    /// The sculk and bottle o' enchanting recipes, into the data the server loads.
    fn data_pack_files(&self, _: &dyn Host, event: &mut DataPackFilesEvent<'_>) {
        event.files.extend(farming::recipes());
        event.files.extend(ground::recipes());
    }

    /// A grown crop right-clicked: harvested and replanted.
    fn right_click_block(&self, host: &mut dyn Host, event: &mut RightClickBlockEvent) -> Flow {
        farming::right_click(host, event.pos, &mut percent)
    }

    /// A diamond item and a netherite ingot in an anvil: its netherite kind, no template needed.
    fn anvil_update(&self, host: &mut dyn Host, event: &mut AnvilUpdateEvent<'_>) -> Flow {
        if let Some(out) = anvil::upgrade(event.left, event.right, |i| host.max_stack_size(i).is_some()) {
            *event.output = Some(out);
            event.xp_cost = anvil::XP_COST;
            event.material_cost = 1;
        }
        Flow::Continue
    }

    /// A server player's enchanting offers, kept to charge half their level.
    fn prepare_item_enchant(&self, _: &mut dyn Host, event: &mut PrepareItemEnchantEvent<'_>) {
        enchant::offered(event.player, *event.costs);
    }

    /// A server player enchanted: half the level asked, less the 1 to 3 levels already taken.
    fn enchant_item(&self, host: &mut dyn Host, event: &mut EnchantItemEvent<'_>) {
        let owed = enchant::still_owed(event.player, event.levels);
        if owed > 0 {
            host.give_levels(event.player, -owed);
        }
    }

    /// On a client: the baby looks its server worked out.
    fn net_from_server(&self, _: &mut dyn Host, event: &mut NetFromServerEvent<'_>) {
        if event.channel == babies::CHANNEL {
            babies::received(event.data);
        }
    }

    /// Deepslate digs only 15% slower than the stone it stands for. Asked on both sides, alike.
    fn break_speed(&self, host: &dyn Host, event: &mut BreakSpeedEvent) {
        if let Some(faster) = deepslate_speedup(&host.block(event.pos)) {
            event.speed *= faster;
        }
    }

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
            v.push((event.entity, event.kind.to_string()));
        }
        Flow::Continue
    }

    /// A chunk loaded: cleared of leaf litter, wildflowers and pink petals on a later tick, once.
    fn chunk_load(&self, _: &mut dyn Host, event: &mut ChunkLoadEvent<'_>) {
        ground::loaded(event.dimension, event.chunk, event.new);
    }

    fn level_tick_post(&self, host: &mut dyn Host, event: &mut LevelTickPostEvent<'_>) {
        ground::tick(host, event.dimension);
        let n = SCAN.fetch_add(1, Ordering::Relaxed);
        if n % 10 == 0 {
            babies::scan(host, event.dimension);
        }
        // Every mob with a newer look, every two seconds: those given one after they joined (their
        // biome's variant picked once they were in the world) as well.
        if n % 40 == 5 {
            for (id, kind, _) in host.level_entities(event.dimension) {
                if has_variant(&kind) {
                    revert_look(host, id, &kind);
                }
            }
        }
        if let Ok(mut c) = CONFIG.lock() {
            c.refresh(|| host.config_dir().ok());
        }
        let ids: Vec<(u32, String)> = TO_REVERT.lock().map(|mut v| std::mem::take(&mut *v)).unwrap_or_default();
        for (id, kind) in ids {
            revert_look(host, id, &kind);
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
    fn deepslate_digs_fifteen_percent_slower_than_stone() {
        // Effective hardness: vanilla's divided by the speed-up.
        let eff = |b: &str, h: f32| h / deepslate_speedup(b).unwrap();
        assert!((eff("minecraft:deepslate", 3.0) - 1.725).abs() < 1e-4);
        assert!((eff("minecraft:cobbled_deepslate", 3.5) - 2.3).abs() < 1e-4);
        assert!((eff("minecraft:deepslate_diamond_ore", 4.5) - 3.45).abs() < 1e-4);
        assert!((eff("minecraft:deepslate_bricks", 3.5) - 1.725).abs() < 1e-4);
        assert_eq!(deepslate_speedup("minecraft:stone"), None);
        assert_eq!(deepslate_speedup("minecraft:infested_deepslate"), None);
        assert_eq!(deepslate_speedup("minecraft:reinforced_deepslate"), None);
    }

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
