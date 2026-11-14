//! No leaf litter, wildflowers or pink petals on the ground.
//!
//! New terrain never gets them (the mixins `no-sculk-placed` and `no-sculk-feature`, which also
//! take out the sculk). Terrain generated before is cleared of them once, chunk by chunk, as each
//! chunk loads: the chunk then remembers it was cleared (an attachment saved with it), so leaf
//! litter and flowers a player makes in a furnace and places later stay where they are put.
//!
//! Leaf litter is smelted from any leaves (vanilla's recipe); wildflowers from a flower and pink
//! petals from a pink tulip (`recipes`).

use std::sync::Mutex;

use horizon_client::mod_api::{
    AttachmentCodec, AttachmentError, AttachmentType, Compound, Holder, Host, NativeBlockEdit, NativeBlockRun, Tag,
};

/// The blocks taken off the ground.
pub const GONE: [&str; 3] = ["minecraft:leaf_litter", "minecraft:wildflowers", "minecraft:pink_petals"];

/// Chunks cleared at most per level tick, so a burst of loading doesn't stall a tick.
const PER_TICK: usize = 16;

/// Chunks loaded and not yet looked at, with their dimension and whether they were just generated.
static LOADED: Mutex<Vec<(String, [i32; 2], bool)>> = Mutex::new(Vec::new());

fn encode(_: &bool) -> Result<Compound, AttachmentError> {
    Ok([("cleared".into(), Tag::Byte(1))].into())
}

fn decode(_: &Compound) -> Result<bool, AttachmentError> {
    Ok(true)
}

/// Set on a chunk once its leaf litter and flowers are gone; saved with the chunk.
static CLEARED: AttachmentType<bool> = AttachmentType {
    namespace: "misogynist_unexpansion",
    name: "ground_cleared",
    default: || false,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: false,
    sync: false,
};

/// A chunk came into the world: it is looked at on a later tick, as the hook asks.
pub fn loaded(dimension: &str, chunk: [i32; 2], new: bool) {
    if let Ok(mut l) = LOADED.lock() {
        l.push((dimension.to_owned(), chunk, new));
    }
}

/// The edit that turns every leaf litter, wildflower and pink petal block in a chunk's columns to air.
pub fn clearing(chunk: [i32; 2], min_y: i32, height: u32) -> NativeBlockEdit {
    let mut runs = Vec::with_capacity(256 * GONE.len());
    for dx in 0..16 {
        for dz in 0..16 {
            for only in 0..GONE.len() as u32 {
                runs.push(NativeBlockRun {
                    x: chunk[0] * 16 + dx,
                    z: chunk[1] * 16 + dz,
                    y: min_y,
                    height,
                    surface: None,
                    block: 0,
                    only: Some(only),
                });
            }
        }
    }
    NativeBlockEdit { palette: vec!["minecraft:air".into()], filters: GONE.iter().map(|b| (*b).to_owned()).collect(), runs }
}

/// The level `dimension` is ticking: its chunks that loaded since are cleared, once each.
pub fn tick(host: &mut dyn Host, dimension: &str) {
    let mine: Vec<([i32; 2], bool)> = {
        let Ok(mut l) = LOADED.lock() else { return };
        let mut taken = Vec::new();
        l.retain(|(d, c, n)| {
            if d == dimension && taken.len() < PER_TICK {
                taken.push((*c, *n));
                false
            } else {
                true
            }
        });
        taken
    };
    if mine.is_empty() {
        return;
    }
    let Some((min_y, height)) = host.level(dimension).and_then(|l| host.level_height(l)) else { return };
    for (chunk, new) in mine {
        let holder = Holder::Chunk(chunk);
        if CLEARED.get(host, holder).unwrap_or(true) {
            continue;
        }
        // A chunk just generated has none: it is only marked.
        if !new {
            let _ = host.edit_blocks(dimension, &clearing(chunk, min_y, height.max(0) as u32));
        }
        let _ = CLEARED.set(host, holder, true);
    }
}

/// The furnace recipes, as (data pack key, JSON): wildflowers from any flower but the pink tulip,
/// pink petals from the pink tulip. Leaf litter from leaves is vanilla's own.
pub fn recipes() -> Vec<(String, Vec<u8>)> {
    let smelt = |from: &[&str], result: &str| {
        format!(
            r#"{{"type":"minecraft:smelting","category":"misc","cookingtime":200,"experience":0.1,"ingredient":[{}],"result":{{"id":"{result}"}}}}"#,
            from.iter().map(|i| format!("\"{i}\"")).collect::<Vec<_>>().join(",")
        )
    };
    let flowers = [
        "minecraft:dandelion",
        "minecraft:poppy",
        "minecraft:blue_orchid",
        "minecraft:allium",
        "minecraft:azure_bluet",
        "minecraft:red_tulip",
        "minecraft:orange_tulip",
        "minecraft:white_tulip",
        "minecraft:oxeye_daisy",
        "minecraft:cornflower",
        "minecraft:lily_of_the_valley",
    ];
    [
        ("wildflowers_from_smelting", smelt(&flowers, "minecraft:wildflowers")),
        ("pink_petals_from_smelting", smelt(&["minecraft:pink_tulip"], "minecraft:pink_petals")),
    ]
    .into_iter()
    .map(|(name, json)| (format!("recipe/misogynist_unexpansion:{name}"), json.into_bytes()))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_client::mod_api::serde_json::{self, Value};

    #[test]
    fn a_chunk_is_cleared_in_every_column_for_each_block() {
        let e = clearing([2, -1], -64, 384);
        assert_eq!(e.runs.len(), 256 * 3);
        assert_eq!(e.palette, vec!["minecraft:air".to_owned()]);
        assert!(e.runs.iter().all(|r| (32..48).contains(&r.x) && (-16..0).contains(&r.z) && r.y == -64 && r.height == 384));
        assert!(e.runs.iter().all(|r| r.only.is_some_and(|o| (o as usize) < e.filters.len())));
    }

    #[test]
    fn the_flowers_are_smelted() {
        for (key, json) in recipes() {
            let v: Value = serde_json::from_slice(&json).unwrap();
            assert_eq!(v["type"], "minecraft:smelting", "{key}");
            assert!(v["ingredient"].as_array().is_some_and(|a| !a.is_empty()));
        }
    }

    /// The worldgen edit the mixins make, run here on vanilla's features.
    mod worldgen {
        use horizon_client::mod_api::serde_json;
        include!("../../no_sculk_edit.rs");

        #[test]
        fn leaf_litter_patches_place_air() {
            let f = r#"{"type":"minecraft:simple_block","config":{"to_place":{"type":"minecraft:weighted","entries":[{"data":"minecraft:leaf_litter","weight":1},{"data":{"id":"minecraft:leaf_litter","properties":{"facing":"east","segment_amount":"1"}},"weight":1}]}}}"#;
            let out = edit(f).unwrap();
            assert!(!out.contains("leaf_litter"), "{out}");
            assert!(out.contains(r#""data":"minecraft:air""#));
            assert!(out.contains(r#""id":"minecraft:air""#));
        }

        #[test]
        fn trees_drop_only_their_leaf_litter() {
            let f = r#"{"type":"minecraft:tree","config":{"decorators":[{"type":"minecraft:place_on_ground","block_state_provider":{"type":"minecraft:weighted","entries":[{"data":"minecraft:leaf_litter","weight":1}]}},{"type":"minecraft:beehive","probability":0.002}],"trunk_provider":{"type":"minecraft:simple_state_provider","state":{"Name":"minecraft:oak_log"}}}}"#;
            let out = edit(f).unwrap();
            assert!(!out.contains("leaf_litter") && out.contains("beehive") && out.contains("oak_log"), "{out}");
        }

        #[test]
        fn flowers_and_petals_go_and_others_stay() {
            let f = r#"{"type":"minecraft:flower","config":{"feature":{"type":"minecraft:simple_block","config":{"to_place":{"type":"minecraft:simple_state_provider","state":{"Name":"minecraft:pink_petals","Properties":{"flower_amount":"2"}}}}},"other":{"data":"minecraft:wildflowers"},"keep":{"data":"minecraft:poppy"}}}"#;
            let out = edit(f).unwrap();
            assert!(!out.contains("pink_petals") && !out.contains("wildflowers") && out.contains("poppy"), "{out}");
            assert!(!out.contains("Properties"), "{out}");
        }

        #[test]
        fn a_feature_named_like_a_block_is_left_alone() {
            assert_eq!(edit(r#"{"type":"minecraft:random_selector","default":"minecraft:oak_leaf_litter","features":[]}"#), None);
        }

        #[test]
        fn sculk_still_goes() {
            let out = edit(r#"{"type":"minecraft:sculk_patch","config":{"spread_rounds":3,"growth_rounds":1}}"#).unwrap();
            assert!(out.contains(r#""spread_rounds":0"#));
        }
    }
}
