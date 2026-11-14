// What powder snow asks of the player, as if leather boots were always on: it holds them up, as
// it did every block of snow before (the player still sinks in when falling onto it from high up).
fn powder_snow_walk(value: crate::collisions::PowderSnowSubject) -> crate::collisions::PowderSnowSubject {
    crate::collisions::PowderSnowSubject { leather_boots: true, ..value }
}
