# Minecraft Misogynist Unexpansion

A [Horizon](https://playhorizon.dev) mod that takes the game back towards how it was.

## What it does

- **Takes out mobs.** By default: dolphins, foxes, sniffers, copper golems, allays, turtles, bees,
  parrots, husks, strays, polar bears, pandas, camels, llamas, phantoms, frogs, glow squids,
  striders, camel husks, armadillos, axolotls, happy ghasts, sulfur cubes, nautiluses, trader llamas,
  zombie nautiluses, bogged, parched, breezes, creakings, tadpoles and goats. None spawn, and those
  already in a world are gone when they load.
- **The original farm animals.** Cows, pigs and chickens only come in their original look: no cold
  or warm ones (no black chickens or light brown cows), old ones included.
- **The old babies.** Calves, piglets and chicks are drawn as before: the grown model and texture,
  the body half size and the head kept big.
- **What the removed mobs gave, made another way:**

  | Item | How |
  |---|---|
  | Honey bottle | slime ball, sugar and a glass bottle (shapeless) |
  | Honeycomb | a honey bottle, smelted |
  | Glow ink sac | ink sac and glowstone dust (shapeless) |
  | Ochre froglight | glowstone ringed with eight slime balls |
  | Verdant froglight | a sea lantern ringed with eight slime balls |
  | Pearlescent froglight | a shroomlight ringed with eight slime balls |
  | Potion of Dolphin's Grace (3:00) | glass bottle, cod and prismarine crystals (shapeless) |
  | Fish eggs (turtle eggs) | nine tropical fish |
  | Fish scales (turtle scutes) | dropped by tropical fish, nine times in ten |

## Choosing which mobs go

`removed_mobs.cfg`, in the mod's config folder, is written the first time the game runs. It lists
only the mobs above, each `removed` or `kept`:

```
dolphin = removed
fox = kept
```

A saved change takes hold within ten seconds. Nothing else (zombies, creepers...) can be removed.

## Two parts

Install both, then press Build in the launcher:

- **`misogynist_unexpansion_native`** (`.hnmod`): the code. A native mod: it runs with your
  account's permissions, so the launcher asks before building it.
- **`misogynist_unexpansion`** (`.hmod`): the recipes and the potion.

## Not possible yet

- A brewing recipe: a mod can't add one (they live in vanilla's own folder), so the Potion of
  Dolphin's Grace is crafted.
- Renaming vanilla items everywhere: fish scales and fish eggs are named in their tooltips; the
  name shown above the hotbar is still vanilla's.
- Old babies for sheep, mooshrooms, wolves, cats and the rest: only calves, piglets and chicks so far.

## Building

`gen_content.py <folder>` writes the content mod; `cargo horizon-mod build` packs it. The native
mod is built by the Horizon launcher (`native-build`), pinned to one Horizon revision.

MIT licensed. No Minecraft code or assets are included.
