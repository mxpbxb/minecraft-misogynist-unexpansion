// A scaffolding's distance as the world reads it: a 6, the furthest a scaffolding still stands at,
// reads as 5, so the next scaffolding along is a 6 as well and also stands, however far it goes.
// Scaffolding has no other number to read.
fn endless_scaffolding(table: &StateTable, state: BlockStateId, value: i32) -> i32 {
    if value == 6 && table.name(state) == "minecraft:scaffolding" { 5 } else { value }
}
