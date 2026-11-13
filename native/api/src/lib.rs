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
//!
//! The recipes and the Potion of Dolphin's Grace are the content mod `misogynist_unexpansion`.

mod babies;
mod config;

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use horizon_client::mod_api::{
    EntityJoinLevelEvent, Flow, Hooks, Host, ItemStack, LevelTickPostEvent, LootDropsEvent, MobSubmitEvent, Tag,
    Text, TooltipEvent,
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

impl Hooks for Mod {
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
