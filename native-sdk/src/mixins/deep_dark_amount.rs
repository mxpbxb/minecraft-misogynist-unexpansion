// The deep dark as `worldgen.cfg` sets it: when its amount isn't vanilla's, the biome tables are
// assembled as usual (this hook letting that inner call through), then every deep dark in them,
// vanilla's and each mod region's copy, is made more or less common.
fn deep_dark_amount(
    presets: &mut BTreeMap<String, String>,
    declarations: &BTreeMap<String, String>,
    biomes: &BTreeSet<String>,
) -> std::ops::ControlFlow<Result<(), DecodeError>> {
    use crate::unexpansion_worldgen as w;
    if w::inside() {
        return std::ops::ControlFlow::Continue(());
    }
    let factor = w::settings().deep_dark;
    if (factor - 1.0).abs() < 1e-9 {
        return std::ops::ControlFlow::Continue(());
    }
    let result = w::reentered(|| extend_presets(presets, declarations, biomes));
    if result.is_ok() {
        w::scale_deep_dark(presets, factor);
    }
    std::ops::ControlFlow::Break(result)
}
