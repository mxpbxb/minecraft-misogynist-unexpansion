//! Netherite without a smithing template: a diamond tool, weapon or piece of armour and a netherite
//! ingot in an anvil make its netherite kind, keeping its enchantments, name and wear, as the
//! smithing table would. The anvil works it out on both sides (the server's menu and the window's
//! prediction) from what it is told, so a server and its players agree.

use horizon_client::mod_api::ItemStack;

/// The diamond items that have a netherite kind.
const UPGRADES: [&str; 10] =
    ["sword", "pickaxe", "axe", "shovel", "hoe", "spear", "helmet", "chestplate", "leggings", "boots"];

/// Levels the upgrade costs: the least a take can cost (a free result can't be taken).
pub const XP_COST: i32 = 1;

/// The netherite kind of `item`, when it is one of the diamond items that have one.
pub fn netherite_of(item: &str) -> Option<String> {
    let piece = item.strip_prefix("minecraft:diamond_")?;
    UPGRADES.contains(&piece).then(|| format!("minecraft:netherite_{piece}"))
}

/// What the anvil makes of `left` and `right`: `left` as its netherite kind, when `right` is a
/// netherite ingot. `known` says whether the game has that item.
pub fn upgrade(left: &ItemStack, right: Option<&ItemStack>, known: impl Fn(&str) -> bool) -> Option<ItemStack> {
    let right = right?;
    if &*right.item != "minecraft:netherite_ingot" || right.count == 0 || left.count != 1 {
        return None;
    }
    let to = netherite_of(&left.item).filter(|n| known(n))?;
    let mut out = left.clone();
    out.item = to.into();
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond_gear_becomes_netherite_with_an_ingot() {
        let mut sword = ItemStack::new("minecraft:diamond_sword", 1);
        sword.damage = 40;
        let ingot = ItemStack::new("minecraft:netherite_ingot", 1);
        let out = upgrade(&sword, Some(&ingot), |_| true).unwrap();
        assert_eq!(&*out.item, "minecraft:netherite_sword");
        assert_eq!((out.count, out.damage), (1, 40));
        assert_eq!(netherite_of("minecraft:diamond_boots").as_deref(), Some("minecraft:netherite_boots"));
    }

    #[test]
    fn nothing_else_upgrades() {
        let ingot = ItemStack::new("minecraft:netherite_ingot", 1);
        let diamond = ItemStack::new("minecraft:diamond", 1);
        let iron = ItemStack::new("minecraft:iron_sword", 1);
        let sword = ItemStack::new("minecraft:diamond_sword", 1);
        assert!(upgrade(&diamond, Some(&ingot), |_| true).is_none());
        assert!(upgrade(&iron, Some(&ingot), |_| true).is_none());
        assert!(upgrade(&sword, Some(&diamond), |_| true).is_none());
        assert!(upgrade(&sword, None, |_| true).is_none());
        // An item this game doesn't have (a spear before its update) isn't made.
        let spear = ItemStack::new("minecraft:diamond_spear", 1);
        assert!(upgrade(&spear, Some(&ingot), |i| i != "minecraft:netherite_spear").is_none());
    }
}
