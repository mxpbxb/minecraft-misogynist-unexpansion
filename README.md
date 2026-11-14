# Minecraft Misogynist Unexpansion

A [Horizon](https://playhorizon.dev) mod that takes the game back towards how it was.

## What it does

- **Takes out mobs.** By default: dolphins, foxes, sniffers, copper golems, allays, turtles, bees,
  parrots, husks, strays, polar bears, pandas, camels, llamas, phantoms, frogs, glow squids,
  striders, camel husks, armadillos, axolotls, happy ghasts, sulfur cubes, nautiluses, trader llamas,
  zombie nautiluses, bogged, parched, breezes, creakings, tadpoles and goats. None spawn, and those
  already in a world are gone when they load.
- **No new looks for old mobs.** Cows, pigs and chickens only come in their original look (no cold
  or warm ones), and wolves only as the original wolf with its classic sounds (none of the eight
  newer wolves). Old ones are changed back too, checked every two seconds.
- **The old babies.** Every baby the redesign changed is drawn as before, with the grown model,
  textures and overlays at the old sizes: villagers (half size, in their biome's clothes), zombies,
  husks, drowned, zombie villagers, piglins and zombified piglins (big head, small body), cows,
  mooshrooms, pigs, sheep (with their wool's colour), chickens, wolves (variant, collar), cats
  (variant, collar), ocelots, rabbits, hoglins, zoglins, horses (colour and markings), donkeys,
  mules, skeleton and zombie horses, squid and glow squid. The mobs this mod removes by default keep
  the new look if you bring them back.
- **Fishing as it was.** Treasure can be caught anywhere again, not only in open water (a pond fishes
  like the ocean), and tropical fish come 7 times in 100 instead of 2.
- **No sculk in new terrain.** Sculk patches (the deep dark's sculk floors, sensors and shriekers)
  and loose sculk veins no longer generate; the sculk is crafted instead (below). Terrain already
  generated keeps its sculk, and ancient city buildings still have the sculk built into them.
- **Walk on powder snow.** Powder snow holds you up as if you wore leather boots; falling onto it
  from high up still sinks you in.
- **Right-click harvesting.** Right-clicking a fully grown crop (wheat, carrots, potatoes,
  beetroots, nether wart, cocoa) harvests it and replants it: it drops what breaking it would, less
  the one item it is replanted with (no Fortune). Sneak to place blocks on crops as usual.
- **Key Binds search by key.** The search box on the Key Binds screen finds a binding by the key
  it is on as well as by its name: type `r`, `shift` or `mouse` to see what uses it. The idea is
  [Controlling](https://github.com/jaredlll08/Controlling)'s (Jaredlll08, MIT), written anew for
  Horizon. (Horizon already has Mouse Tweaks built in: Horizon Settings, Extras.)
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
  | Sculk | dirt and a bottle o' enchanting (shapeless) |
  | 4 sculk veins | one sculk |
  | Sculk sensor | `V V` / `SRS`: two sculk veins over sculk, redstone, sculk |
  | Sculk catalyst | `BXB` / `SSS` / `SSS`: bone, bottle o' enchanting, bone over six sculk |
  | Sculk shrieker | `B B` / `SXS` / `SSS`: two bones, then sculk, bottle o' enchanting, sculk, over three sculk (never summons a warden) |
  | Bottle o' enchanting | any mob head or skull: skeleton, wither skeleton, zombie, creeper, piglin or dragon |

  The sculk and bottle o' enchanting recipes come from the native mod, so the content mod's file
  stays the same as servers that check it know.

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
- **`misogynist_unexpansion`** (`.hmod`): the first recipes and the potion.

## Not possible yet

- A brewing recipe: a mod can't add one (they live in vanilla's own folder), so the Potion of
  Dolphin's Grace is crafted.
- Renaming vanilla items everywhere: fish scales and fish eggs are named in their tooltips; the
  name shown above the hotbar is still vanilla's.
- Babies are drawn without held items or armour, and a sitting or lying wolf or cat baby stands.

## Building

`gen_content.py <folder>` writes the content mod; `cargo horizon-mod build` packs it. The native
mod is built by the Horizon launcher (`native-build`), pinned to one Horizon revision.

MIT licensed. No Minecraft code or assets are included.
