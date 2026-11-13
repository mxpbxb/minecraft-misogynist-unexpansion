//! Minecraft Misogynist Unexpansion: the game as it was, more or less.
//!
//! - The mobs in `removed_mobs.cfg` (dolphins, foxes, bees, goats and the rest of the newer ones by
//!   default) never join a world: none spawn, and those already in one are gone when they load.
//! - Cows, pigs and chickens come only in their original (temperate) look, old ones included.
//! - Calves, piglets and chicks look as they did: the grown model and texture, the body half size
//!   and the head kept big, as `AgeableListModel` drew them.
//! - Tropical fish drop fish scales (turtle scutes), nine times in ten; turtle scutes and eggs are
//!   called fish scales and fish eggs.
//!
//! The recipes and the Potion of Dolphin's Grace are the content mod `misogynist_unexpansion`.

mod config;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use horizon_client::mod_api::{
    EntityJoinLevelEvent, Flow, Hooks, Host, ItemStack, LevelTickPostEvent, LootDropsEvent, MobSubmitEvent,
    ModelPart, Tag, Text, TooltipEvent,
};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod)
}

struct Mod;

static CONFIG: LazyLock<Mutex<config::Loaded>> = LazyLock::new(|| Mutex::new(config::Loaded::new()));
/// Cows, pigs and chickens that just joined a world, to set to the temperate variant next tick.
static TO_REVERT: Mutex<Vec<u32>> = Mutex::new(Vec::new());
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

// ---- Old babies ----

/// A baby drawn the old way: its grown model and texture, and `AgeableListModel`'s numbers for it
/// (`babyYHeadOffset`, `babyZHeadOffset`; every one of these halves the body and keeps the head).
struct OldBaby {
    kind: &'static str,
    layer: &'static str,
    texture: &'static str,
    head_y: f32,
    head_z: f32,
    legs: Legs,
}

#[derive(Clone, Copy)]
enum Legs {
    Four,
    Chicken,
}

const OLD_BABIES: [OldBaby; 3] = [
    OldBaby { kind: "minecraft:cow", layer: "minecraft:cow#main", texture: "entity/cow/cow_temperate", head_y: 10.0, head_z: 4.0, legs: Legs::Four },
    OldBaby { kind: "minecraft:pig", layer: "minecraft:pig#main", texture: "entity/pig/pig_temperate", head_y: 4.0, head_z: 4.0, legs: Legs::Four },
    OldBaby {
        kind: "minecraft:chicken",
        layer: "minecraft:chicken#main",
        texture: "entity/chicken/chicken_temperate",
        head_y: 5.0,
        head_z: 2.0,
        legs: Legs::Chicken,
    },
];

/// The grown models, baked once.
static BAKED: LazyLock<Mutex<HashMap<&'static str, ModelPart>>> = LazyLock::new(Default::default);

fn set(root: &mut ModelPart, name: &str, f: impl FnOnce(&mut ModelPart)) {
    if let Some(p) = root.get_child_mut(name) {
        f(p);
    }
}

/// Shows only the head (`head` true), or only the rest.
fn show(root: &mut ModelPart, head: bool) {
    for (name, part) in root.children_mut() {
        part.visible = (name == "head") == head;
    }
}

impl Mod {
    fn old_baby(&self, host: &dyn Host, event: &mut MobSubmitEvent<'_>) -> Flow {
        let mob = event.mob;
        if !mob.baby {
            return Flow::Continue;
        }
        let Some(baby) = OLD_BABIES.iter().find(|b| b.kind == mob.kind) else { return Flow::Continue };
        let Ok(mut baked) = BAKED.lock() else { return Flow::Continue };
        if !baked.contains_key(baby.layer) {
            let Some(part) = host.bake_layer(baby.layer) else { return Flow::Continue };
            baked.insert(baby.layer, part);
        }
        let Some(root) = baked.get_mut(baby.layer) else { return Flow::Continue };
        if root.get_child("head").is_none() {
            return Flow::Continue;
        }
        root.reset_all_poses();
        // QuadrupedModel.setupAnim and ChickenModel.setupAnim.
        set(root, "head", |p| {
            p.x_rot = mob.pitch.to_radians();
            p.y_rot = mob.head_yaw.to_radians();
        });
        let swing = (mob.walk_position * 0.6662).cos() * 1.4 * mob.walk_speed;
        match baby.legs {
            Legs::Four => {
                for (name, sign) in [("right_hind_leg", 1.0), ("left_front_leg", 1.0), ("left_hind_leg", -1.0), ("right_front_leg", -1.0)] {
                    set(root, name, |p| p.x_rot = swing * sign);
                }
            }
            Legs::Chicken => {
                set(root, "right_leg", |p| p.x_rot = swing);
                set(root, "left_leg", |p| p.x_rot = -swing);
                // Flapping while off the ground.
                let flap = if mob.on_ground { 0.0 } else { ((mob.age * 1.3).sin() + 1.0) * 0.6 };
                set(root, "right_wing", |p| p.z_rot = flap);
                set(root, "left_wing", |p| p.z_rot = -flap);
            }
        }
        let tint = if mob.hurt_time > 0 || mob.death_time > 0 { 0xFF_99_99 } else { 0xFF_FF_FF };
        let draw = &mut *event.draw;
        draw.push();
        draw.scale([1.0, -1.0, -1.0]);
        draw.translate([0.0, -1.501, 0.0]);
        // AgeableListModel.renderToBuffer for the young: the head full size, moved down and forward;
        // the body at half size, moved down to the ground.
        show(root, true);
        draw.push();
        draw.translate([0.0, baby.head_y / 16.0, baby.head_z / 16.0]);
        host.draw_model_part(draw, root, baby.texture, tint);
        draw.pop();
        show(root, false);
        draw.push();
        draw.scale([0.5, 0.5, 0.5]);
        draw.translate([0.0, 24.0 / 16.0, 0.0]);
        host.draw_model_part(draw, root, baby.texture, tint);
        draw.pop();
        draw.pop();
        for (_, part) in root.children_mut() {
            part.visible = true;
        }
        Flow::Cancel
    }
}

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

    fn level_tick_post(&self, host: &mut dyn Host, _: &mut LevelTickPostEvent<'_>) {
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
        self.old_baby(host, event)
    }
}
