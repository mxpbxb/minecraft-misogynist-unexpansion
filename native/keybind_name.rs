// A translation, as the game looks it up: while the Key Binds screen lays itself out, a binding's
// name carries its key too, so the screen's search finds it by either.
fn keybind_name(key: &str, value: String) -> String {
    match crate::keybind_search::extra(key) {
        Some(extra) => format!("{value} {extra}"),
        None => value,
    }
}
