// Misogynist Unexpansion: no sculk in new terrain. A feature tree, fully expanded, with every
// sculk patch given no rounds of spreading or growth and every sculk-vein multiface growth
// allowed on no face: neither then places anything. None when the tree has neither.

pub fn edit(expanded: &str) -> Option<String> {
    if !expanded.contains("sculk") {
        return None;
    }
    let mut value: serde_json::Value = serde_json::from_str(expanded).ok()?;
    walk(&mut value).then(|| value.to_string())
}

fn short(s: &str) -> &str {
    s.trim_start_matches("minecraft:")
}

/// The changes made under `v`: whether there were any.
fn walk(v: &mut serde_json::Value) -> bool {
    let mut changed = false;
    match v {
        serde_json::Value::Object(map) => {
            let kind = map.get("type").and_then(|t| t.as_str()).map(|t| short(t).to_owned());
            match kind.as_deref() {
                Some("sculk_patch") => {
                    changed |= zero(map);
                    if let Some(serde_json::Value::Object(config)) = map.get_mut("config") {
                        changed |= zero(config);
                    }
                }
                Some("multiface_growth") => {
                    changed |= no_faces(map);
                    if let Some(serde_json::Value::Object(config)) = map.get_mut("config") {
                        changed |= no_faces(config);
                    }
                }
                _ => {}
            }
            for child in map.values_mut() {
                changed |= walk(child);
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                changed |= walk(child);
            }
        }
        _ => {}
    }
    changed
}

fn zero(map: &mut serde_json::Map<String, serde_json::Value>) -> bool {
    let mut changed = false;
    for key in ["spread_rounds", "growth_rounds"] {
        if map.get(key).and_then(|n| n.as_i64()).is_some_and(|n| n != 0) {
            map.insert(key.to_owned(), serde_json::Value::from(0));
            changed = true;
        }
    }
    changed
}

fn no_faces(map: &mut serde_json::Map<String, serde_json::Value>) -> bool {
    if map.get("block").and_then(|b| b.as_str()).is_none_or(|b| short(b) != "sculk_vein") {
        return false;
    }
    let mut changed = false;
    for key in ["can_place_on_ceiling", "can_place_on_floor", "can_place_on_wall"] {
        if map.get(key).and_then(|f| f.as_bool()) == Some(true) {
            map.insert(key.to_owned(), serde_json::Value::Bool(false));
            changed = true;
        }
    }
    changed
}
