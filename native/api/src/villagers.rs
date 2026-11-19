//! Villagers picked up and carried, as Easy Villagers does it (by henkelmax): sneak and right-click
//! a villager to pick it up as an item, a stack of one named "Villager" ("Baby Villager" for a baby,
//! its own name if it has one) that tells its profession and level, and use the item on a block to
//! set it down again, just as it was: trades, experience, gossip, its job and home.
//!
//! The item is a villager spawn egg holding the villager's saved data in its custom data. This mod
//! answers every use of it, so the egg's own behaviour (a new villager, a baby from a villager, a
//! spawner turned into a villager spawner) never runs.

use horizon_client::mod_api::{
    Compound, EquipmentSlot, Flow, Host, InteractEntityEvent, ItemStack, NativeServerPlayer, Tag, Text, UseItemEvent,
    UseItemOnEvent,
};

pub const KIND: &str = "minecraft:villager";
pub const ITEM: &str = "minecraft:villager_spawn_egg";
const DATA: &str = "minecraft:custom_data";
/// Where in the custom data the villager is.
const KEY: &str = "misogynist_unexpansion:villager";

/// What says where the villager was and who it was in the world, not carried with it: it is set
/// down somewhere else, and comes back as a new entity (the game gives it a new id).
const LEFT_BEHIND: [&str; 11] = [
    "Pos",
    "Motion",
    "UUID",
    "OnGround",
    "FallDistance",
    "Fire",
    "PortalCooldown",
    "SleepingX",
    "SleepingY",
    "SleepingZ",
    "leash",
];

fn get<'a>(c: &'a Compound, key: &str) -> Option<&'a Tag> {
    c.iter().find(|(k, _)| k.to_string_lossy() == key).map(|(_, v)| v)
}

fn string(tag: Option<&Tag>) -> Option<String> {
    match tag? {
        Tag::String(s) => Some(s.to_string_lossy()),
        _ => None,
    }
}

fn int(tag: Option<&Tag>) -> Option<i32> {
    match tag? {
        Tag::Byte(v) => Some(i32::from(*v)),
        Tag::Short(v) => Some(i32::from(*v)),
        Tag::Int(v) => Some(*v),
        _ => None,
    }
}

/// The villager's data as it is carried: all of it but where and who it was in the world.
pub fn carried(nbt: &Compound) -> Compound {
    let mut c = nbt.clone();
    c.retain(|k, _| !LEFT_BEHIND.contains(&k.to_string_lossy().as_str()));
    c
}

pub fn is_baby(nbt: &Compound) -> bool {
    int(get(nbt, "Age")).is_some_and(|age| age < 0)
}

/// A villager can't be picked up asleep in its bed or dead, as in Easy Villagers.
pub fn can_pick_up(nbt: &Compound) -> bool {
    let asleep = get(nbt, "SleepingX").is_some();
    let dead = match get(nbt, "Health") {
        Some(Tag::Float(bits)) => f32::from_bits(*bits) <= 0.0,
        _ => false,
    };
    !asleep && !dead
}

/// `minecraft:master_mason`'s "Master Mason".
fn title(id: &str) -> String {
    let bare = id.rsplit(':').next().unwrap_or(id);
    bare.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

const LEVELS: [&str; 5] = ["Novice", "Apprentice", "Journeyman", "Expert", "Master"];

/// "Farmer (Apprentice)": the villager's profession and level, for the item's tooltip. None for a
/// villager without a profession.
pub fn profession_line(nbt: &Compound) -> Option<String> {
    let Some(Tag::Compound(data)) = get(nbt, "VillagerData") else { return None };
    let profession = string(get(data, "profession"))?;
    if profession.ends_with(":none") || profession == "none" {
        return None;
    }
    let level = int(get(data, "level")).unwrap_or(1).clamp(1, 5) as usize;
    Some(format!("{} ({})", title(&profession), LEVELS[level - 1]))
}

/// The item a picked-up villager becomes.
pub fn item_for(nbt: &Compound) -> ItemStack {
    let mut stack = ItemStack::new(ITEM, 1);
    let data = stack.data_mut();
    let mut inner = Compound::new();
    inner.insert(KEY.into(), Tag::Compound(carried(nbt)));
    data.set_other(DATA, Some(Tag::Compound(inner)));
    let name = if is_baby(nbt) { "Baby Villager" } else { "Villager" };
    data.set_other("minecraft:item_name", Some(Tag::String(name.into())));
    data.set_other("minecraft:max_stack_size", Some(Tag::Int(1)));
    if let Some(line) = profession_line(nbt) {
        data.set_other("minecraft:lore", Some(Tag::List(vec![Tag::String(line.as_str().into())])));
    }
    if let Some(custom) = get(nbt, "CustomName") {
        data.set_other("minecraft:custom_name", Some(custom.clone()));
    }
    stack
}

/// The villager an item carries, if it is one.
pub fn villager_in(stack: &ItemStack) -> Option<Compound> {
    if stack.item.as_ref() != ITEM {
        return None;
    }
    let Tag::Compound(data) = stack.data()?.other(DATA)? else { return None };
    match get(data, KEY)? {
        Tag::Compound(v) => Some(v.clone()),
        _ => None,
    }
}

/// Where a villager set down on the block at `pos`, clicked on `face`, stands: on the clicked spot
/// when it has nothing to stand against (grass, flowers, snow), else in front of the face.
pub fn set_down_at(pos: [i32; 3], face: [i32; 3], passable: bool) -> [f64; 3] {
    let b = if passable { pos } else { [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]] };
    [f64::from(b[0]) + 0.5, f64::from(b[1]), f64::from(b[2]) + 0.5]
}

/// Sneak, right-click a villager: it is picked up. Any right-click on a mob with a carried villager
/// does nothing (a spawn egg's would make a baby).
pub fn interact_entity(host: &mut dyn Host, event: &InteractEntityEvent<'_>) -> Flow {
    if event.stack.is_some_and(|s| villager_in(s).is_some()) {
        return Flow::Cancel;
    }
    if event.kind != KIND || !host.sneaking() {
        return Flow::Continue;
    }
    let Ok(id) = u32::try_from(event.entity) else { return Flow::Continue };
    let Some(entity) = host.entity(id) else { return Flow::Continue };
    let Some(nbt) = host.entity_nbt(entity) else { return Flow::Continue };
    if !can_pick_up(&nbt) {
        return Flow::Continue;
    }
    let Some(player) = host.acting_player() else { return Flow::Continue };
    let Some(dimension) = host.entity_level(entity).map(|l| l.dimension().to_owned()) else {
        return Flow::Continue;
    };
    let item = item_for(&nbt);
    // The item first: with no room for it, the villager stays where it is.
    let in_hand = event.stack.is_none() && to_main_hand(host, &player, &item);
    if !in_hand && host.give_stack(player.uuid, item.clone()).is_some() {
        host.action_bar(player.uuid, Text::literal("No room in your inventory for the villager"));
        return Flow::Cancel;
    }
    // Then the villager goes; if it can't, the item does, so it is never in two places.
    if !host.remove_mob(&dimension, id) {
        take_one(host, &item);
    }
    Flow::Cancel
}

/// Into the player's empty main hand, as Easy Villagers does; false if it didn't take.
fn to_main_hand(host: &mut dyn Host, player: &NativeServerPlayer, item: &ItemStack) -> bool {
    let Some(living) = u32::try_from(player.entity_id).ok().and_then(|id| host.entity(id)).and_then(|e| host.living(e))
    else {
        return false;
    };
    living.set_equipment(host, EquipmentSlot::MainHand, Some(item.clone()))
        && host.equipment(EquipmentSlot::MainHand).as_ref() == Some(item)
}

/// One of the carried villager the player is using, out of the slot that holds it.
fn take_one(host: &mut dyn Host, stack: &ItemStack) -> bool {
    for slot in (0..36).chain([-106]) {
        let Some(s) = host.carried_stack(slot) else { continue };
        if s.item == stack.item && s.extra == stack.extra && s.count > 0 {
            s.count -= 1;
            return true;
        }
    }
    false
}

/// A carried villager used on a block is set down there.
pub fn use_item_on(host: &mut dyn Host, event: &UseItemOnEvent<'_>) -> Flow {
    let Some(nbt) = villager_in(event.stack) else { return Flow::Continue };
    let Some(player) = host.acting_player() else { return Flow::Cancel };
    let passable = host.block_collision(event.pos).is_empty();
    let at = set_down_at(event.pos, event.face, passable);
    let survival = player.game_mode != 1;
    if survival && !take_one(host, event.stack) {
        return Flow::Cancel;
    }
    if host.summon(&player.dimension, KIND, at, nbt).is_none() {
        if survival {
            let mut back = event.stack.clone();
            back.count = 1;
            host.give_stack(player.uuid, back);
        }
        host.action_bar(player.uuid, Text::literal("There is no room for the villager here"));
    }
    Flow::Cancel
}

/// Used in the air (or on water) it does nothing: an egg's use would make a new villager.
pub fn use_item(event: &UseItemEvent<'_>) -> Flow {
    if villager_in(event.stack).is_some() { Flow::Cancel } else { Flow::Continue }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn villager(age: i32, profession: &str, level: i32) -> Compound {
        let mut data = Compound::new();
        data.insert("profession".into(), Tag::String(profession.into()));
        data.insert("level".into(), Tag::Int(level));
        data.insert("type".into(), Tag::String("minecraft:plains".into()));
        let mut c = Compound::new();
        c.insert("Age".into(), Tag::Int(age));
        c.insert("VillagerData".into(), Tag::Compound(data));
        c.insert("Pos".into(), Tag::List(vec![Tag::Double(1f64.to_bits()); 3]));
        c.insert("UUID".into(), Tag::IntArray(vec![1, 2, 3, 4]));
        c.insert("Xp".into(), Tag::Int(42));
        c.insert("Health".into(), Tag::Float(20f32.to_bits()));
        c
    }

    #[test]
    fn the_item_carries_the_villager_and_comes_back_as_it_was() {
        let v = villager(0, "minecraft:farmer", 2);
        let item = item_for(&v);
        assert_eq!(item.item.as_ref(), ITEM);
        assert_eq!(item.count, 1);
        let back = villager_in(&item).expect("a villager");
        assert_eq!(get(&back, "Xp"), Some(&Tag::Int(42)));
        assert!(get(&back, "VillagerData").is_some());
        // Not where or who it was.
        assert!(get(&back, "Pos").is_none() && get(&back, "UUID").is_none());
    }

    #[test]
    fn named_by_age_with_its_profession_and_level() {
        let d = |item: &ItemStack, k: &str| item.data().and_then(|d| d.other(k)).cloned();
        let grown = item_for(&villager(0, "minecraft:master_mason", 5));
        assert_eq!(d(&grown, "minecraft:item_name"), Some(Tag::String("Villager".into())));
        assert_eq!(profession_line(&villager(0, "minecraft:farmer", 2)).as_deref(), Some("Farmer (Apprentice)"));
        assert_eq!(profession_line(&villager(0, "minecraft:master_mason", 9)).as_deref(), Some("Master Mason (Master)"));
        assert_eq!(profession_line(&villager(0, "minecraft:none", 1)), None);
        let baby = item_for(&villager(-24000, "minecraft:none", 1));
        assert_eq!(d(&baby, "minecraft:item_name"), Some(Tag::String("Baby Villager".into())));
        assert_eq!(d(&baby, "minecraft:lore"), None);
        assert_eq!(d(&grown, "minecraft:max_stack_size"), Some(Tag::Int(1)));
    }

    #[test]
    fn not_asleep_or_dead() {
        let mut v = villager(0, "minecraft:none", 1);
        assert!(can_pick_up(&v));
        v.insert("SleepingX".into(), Tag::Int(3));
        assert!(!can_pick_up(&v));
        let mut dead = villager(0, "minecraft:none", 1);
        dead.insert("Health".into(), Tag::Float(0f32.to_bits()));
        assert!(!can_pick_up(&dead));
    }

    #[test]
    fn plain_eggs_and_other_items_are_not_villagers() {
        assert!(villager_in(&ItemStack::new(ITEM, 1)).is_none());
        assert!(villager_in(&ItemStack::new("minecraft:stone", 1)).is_none());
    }

    #[test]
    fn set_down_in_front_of_a_solid_face_or_on_grass() {
        assert_eq!(set_down_at([0, 64, 0], [0, 1, 0], false), [0.5, 65.0, 0.5]);
        assert_eq!(set_down_at([3, 64, -2], [1, 0, 0], false), [4.5, 64.0, -1.5]);
        assert_eq!(set_down_at([0, 65, 0], [0, 1, 0], true), [0.5, 65.0, 0.5]);
    }
}
