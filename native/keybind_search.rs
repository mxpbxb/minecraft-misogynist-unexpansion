// Before the options screens lay themselves out: on Key Binds, each binding's key is noted (its name,
// as "R" or "Left Shift", and its id's words, "f13"), for its search to find the binding by.
fn keybind_arm(ui: &crate::options_ui::OptionsUi, s: &crate::settings::Settings) {
    let keys = (ui.page() == crate::options_ui::Page::KeyBinds).then(|| {
        crate::keybinds::all_mappings()
            .map(|(name, _, _)| {
                let key = s.keys.get(name);
                let words = key.trim_start_matches("key.keyboard.").trim_start_matches("key.").replace('.', " ");
                ((*name).to_owned(), format!("{} {}", crate::lang::get(key), words))
            })
            .collect()
    });
    horizon_client_core::keybind_search::arm(keys);
}
