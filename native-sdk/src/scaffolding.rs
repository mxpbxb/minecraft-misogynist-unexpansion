//! Scaffolding stands however far it reaches (the `endless-scaffolding` mixin), so long as it is
//! joined to scaffolding standing on the ground. When a scaffolding's update tick comes, the
//! scaffolding joined to it is walked; if none of it stands on the ground, all of it is marked as
//! out of reach (distance 7), and the game's own scaffolding rule drops it, as vanilla drops a
//! scaffolding too far from its support.

use mod_api::{BlockPart, Flow, LevelWriteHost, TickBlockEvent};
use std::collections::{HashSet, VecDeque};

pub const BLOCK: &str = "minecraft:scaffolding";
/// The most scaffolding walked; a bigger frame is left standing rather than walked further.
const LIMIT: usize = 4096;

/// What a block is to a scaffolding's support.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Scaffolding,
    /// A block a scaffolding can stand on.
    Ground,
    /// Air, a fluid, grass or anything else replaceable: nothing to stand on.
    Open,
    /// Not loaded: whatever is there, the frame is left alone.
    Unknown,
}

/// The scaffolding joined to the one at `start` (through any side), when none of it stands on the
/// ground: None when some does, when part of it is not loaded, or when there is more than `LIMIT`.
pub fn unsupported(cell: impl Fn([i32; 3]) -> Cell, start: [i32; 3]) -> Option<Vec<[i32; 3]>> {
    if cell(start) != Cell::Scaffolding {
        return None;
    }
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    let mut group = Vec::new();
    while let Some(p @ [x, y, z]) = queue.pop_front() {
        group.push(p);
        match cell([x, y - 1, z]) {
            Cell::Ground | Cell::Unknown => return None,
            _ => {}
        }
        for n in [[x, y - 1, z], [x, y + 1, z], [x + 1, y, z], [x - 1, y, z], [x, y, z + 1], [x, y, z - 1]] {
            match cell(n) {
                Cell::Unknown => return None,
                Cell::Scaffolding if seen.insert(n) => {
                    if seen.len() > LIMIT {
                        return None;
                    }
                    queue.push_back(n);
                }
                _ => {}
            }
        }
    }
    Some(group)
}

fn cell(level: &dyn LevelWriteHost, pos: [i32; 3]) -> Cell {
    if !level.is_loaded(pos) {
        return Cell::Unknown;
    }
    let Some(state) = level.block_state(pos) else { return Cell::Unknown };
    match level.state_block(state) {
        Some(BLOCK) => Cell::Scaffolding,
        Some("minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") => Cell::Open,
        Some(_) if level.block_in_tag(state.block(), "minecraft:replaceable") == Some(true) => Cell::Open,
        Some(_) => Cell::Ground,
        None => Cell::Unknown,
    }
}

pub struct Scaffolding;

impl BlockPart for Scaffolding {
    fn block_subscribed(&self) -> &'static [Option<mod_api::HookId>] {
        &[Some(mod_api::HookId::TickBlock)]
    }

    fn tick_block(&self, level: &mut dyn LevelWriteHost, event: &mut TickBlockEvent) -> Flow {
        let start = [event.pos.x, event.pos.y, event.pos.z];
        let Some(group) = unsupported(|p| cell(&*level, p), start) else { return Flow::Continue };
        let block = event.state.block();
        for p in group {
            // Neighbours and clients told; each is ticked again, and with every neighbour out of
            // reach too the game's rule drops it.
            level.set_block_property(p, "distance", "7", 3);
            if p != start {
                level.schedule_block_tick(p, block, 1);
            }
        }
        Flow::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn world(scaffolds: &[[i32; 3]], ground: &[[i32; 3]]) -> impl Fn([i32; 3]) -> Cell {
        let mut m: HashMap<[i32; 3], Cell> = HashMap::new();
        for p in scaffolds {
            m.insert(*p, Cell::Scaffolding);
        }
        for p in ground {
            m.insert(*p, Cell::Ground);
        }
        move |p| m.get(&p).copied().unwrap_or(Cell::Open)
    }

    /// A column at x 0 on the ground at y 0, and a bridge from its top out to x `len`.
    fn bridge(len: i32) -> Vec<[i32; 3]> {
        let mut s: Vec<[i32; 3]> = (1..=3).map(|y| [0, y, 0]).collect();
        s.extend((1..=len).map(|x| [x, 3, 0]));
        s
    }

    #[test]
    fn a_long_bridge_from_a_grounded_column_stands() {
        let s = bridge(40);
        let w = world(&s, &[[0, 0, 0]]);
        assert_eq!(unsupported(&w, [40, 3, 0]), None);
        assert_eq!(unsupported(&w, [20, 3, 0]), None);
    }

    #[test]
    fn with_its_column_gone_the_whole_bridge_falls() {
        let s: Vec<[i32; 3]> = bridge(40).into_iter().filter(|p| p[0] != 0).collect();
        let w = world(&s, &[[0, 0, 0]]);
        let group = unsupported(&w, [40, 3, 0]).unwrap();
        assert_eq!(group.len(), 40);
    }

    #[test]
    fn a_cut_bridge_keeps_the_grounded_half() {
        let s: Vec<[i32; 3]> = bridge(40).into_iter().filter(|p| *p != [20, 3, 0]).collect();
        let w = world(&s, &[[0, 0, 0]]);
        assert_eq!(unsupported(&w, [19, 3, 0]), None);
        let far = unsupported(&w, [21, 3, 0]).unwrap();
        assert_eq!(far.len(), 20);
        assert!(far.iter().all(|p| p[0] > 20));
    }

    #[test]
    fn standing_on_grass_or_water_is_no_support_but_unloaded_is_left_alone() {
        let w = world(&[[0, 1, 0]], &[]);
        assert!(unsupported(&w, [0, 1, 0]).is_some());
        let unloaded = |p: [i32; 3]| if p == [0, 1, 0] { Cell::Scaffolding } else { Cell::Unknown };
        assert_eq!(unsupported(unloaded, [0, 1, 0]), None);
    }

    #[test]
    fn a_frame_too_big_to_walk_is_left_standing() {
        let s: Vec<[i32; 3]> = (0..=LIMIT as i32 + 10).map(|x| [x, 5, 0]).collect();
        let w = world(&s, &[]);
        assert_eq!(unsupported(&w, [0, 5, 0]), None);
    }
}
