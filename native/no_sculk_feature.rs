// A configured feature, expanded as this function would, then decoded with no sculk in it (see
// `no_sculk::edit`). One without sculk goes on to the function as it is.
fn no_sculk_feature(
    catalog: &FeatureCatalog,
    json: &str,
    blocks: Option<&FeatureBlockBindings<'_>>,
) -> std::ops::ControlFlow<Result<FeatureDefinition, DecodeError>> {
    use std::ops::ControlFlow;
    let Ok(value) = serde_json::from_str::<Box<RawValue>>(json) else { return ControlFlow::Continue(()) };
    let mut work = Expansion { remaining: 65536, active: vec![] };
    let Ok(expanded) = catalog.expand_feature(&value, 0, &mut work) else { return ControlFlow::Continue(()) };
    match crate::no_sculk::edit(&expanded) {
        Some(edited) => ControlFlow::Break(FeatureDefinition::from_catalog_json(&edited, &catalog.empty_tags, blocks)),
        None => ControlFlow::Continue(()),
    }
}
