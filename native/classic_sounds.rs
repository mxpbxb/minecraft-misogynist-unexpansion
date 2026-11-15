// Every pig, cow, chicken, wolf and cat makes the classic sounds of its kind, as before the sound
// variants (big and mini pigs, moody cows, picky chickens, royal cats, the angry, big, cute, grumpy,
// puglin and sad wolves) and before the babies' own sounds: a baby makes the grown sound, at its
// higher voice as it always did.
fn classic_sounds(m: &crate::mobs::Mob, kind: &str) -> std::ops::ControlFlow<Option<&'static str>> {
    use crate::mobs::Species;
    use std::ops::ControlFlow;
    use std::{collections::HashMap, sync::Mutex};
    let name = match m.species() {
        Species::Pig => "pig",
        Species::Cow => "cow",
        Species::Chicken => "chicken",
        Species::Wolf => "wolf",
        Species::Cat => "cat",
        _ => return ControlFlow::Continue(()),
    };
    // A handful of ids in all: each kept as a `&'static str`.
    static IDS: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
    let Ok(mut ids) = IDS.lock() else { return ControlFlow::Continue(()) };
    let ids = ids.get_or_insert_with(HashMap::new);
    let id = format!("entity.{name}.{kind}");
    ControlFlow::Break(Some(*ids.entry(id).or_insert_with_key(|k| Box::leak(k.clone().into_boxed_str()))))
}
