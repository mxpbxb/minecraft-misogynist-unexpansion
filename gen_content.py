"""Writes the content mod `misogynist_unexpansion`: the recipes that stand in for the removed mobs'
drops, the Potion of Dolphin's Grace, the new names of turtle scutes and eggs, and the old water
textures.

usage: gen_content.py <content mod folder>
"""
import json
import math
import random
import shutil
import struct
import sys
import zlib
from pathlib import Path

out = Path(sys.argv[1])
NS, VERSION = "misogynist_unexpansion", "0.2.1"
if out.exists():
    shutil.rmtree(out)
data, assets = out / "data" / NS, out / "assets" / NS


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def shapeless(name, ingredients, result, count=1):
    write(data / "recipe" / f"{name}.json", {
        "type": "minecraft:crafting_shapeless", "category": "misc",
        "ingredients": ingredients, "result": {"id": result, "count": count},
    })


def ringed(name, middle, ring, result):
    """`middle` in the centre of a crafting table, eight of `ring` around it."""
    write(data / "recipe" / f"{name}.json", {
        "type": "minecraft:crafting_shaped", "category": "building",
        "pattern": ["RRR", "RMR", "RRR"], "key": {"R": ring, "M": middle}, "result": {"id": result},
    })


# Honey without bees: slime and sugar in a bottle; honeycomb is honey cooked in a furnace.
shapeless("honey_bottle", ["minecraft:slime_ball", "minecraft:sugar", "minecraft:glass_bottle"], "minecraft:honey_bottle")
write(data / "recipe" / "honeycomb_from_smelting.json", {
    "type": "minecraft:smelting", "category": "misc", "ingredient": "minecraft:honey_bottle",
    "result": {"id": "minecraft:honeycomb"}, "experience": 0.1, "cookingtime": 200,
})
# Glow ink without glow squids.
shapeless("glow_ink_sac", ["minecraft:ink_sac", "minecraft:glowstone_dust"], "minecraft:glow_ink_sac")
# Froglights without frogs: a light block ringed with slime.
ringed("ochre_froglight", "minecraft:glowstone", "minecraft:slime_ball", "minecraft:ochre_froglight")
ringed("verdant_froglight", "minecraft:sea_lantern", "minecraft:slime_ball", "minecraft:verdant_froglight")
ringed("pearlescent_froglight", "minecraft:shroomlight", "minecraft:slime_ball", "minecraft:pearlescent_froglight")
# Fish eggs (turtle eggs, renamed) from nine tropical fish.
write(data / "recipe" / "fish_eggs.json", {
    "type": "minecraft:crafting_shaped", "category": "misc",
    "pattern": ["FFF", "FFF", "FFF"], "key": {"F": "minecraft:tropical_fish"}, "result": {"id": "minecraft:turtle_egg"},
})

# The Potion of Dolphin's Grace: three minutes of it. Crafted, as a mod can't add a brewing recipe
# (they live in vanilla's own folder): a glass bottle, a cod and prismarine crystals.
write(data / "horizon" / "item" / "dolphins_grace_potion.json", {
    "template": "minecraft:potion",
    "max_stack_size": 1,
    "components": {
        "minecraft:potion_contents": {
            "custom_color": 0x60B8E8,
            "custom_effects": [{"id": "minecraft:dolphins_grace", "duration": 3600}],
        },
        "minecraft:item_name": {"translate": f"item.{NS}.dolphins_grace_potion"},
    },
})
write(assets / "items" / "dolphins_grace_potion.json", {"model": {
    "type": "minecraft:model", "model": "minecraft:item/potion",
    "tints": [{"type": "minecraft:potion", "default": -13083194}],
}})
shapeless("dolphins_grace_potion", ["minecraft:glass_bottle", "minecraft:cod", "minecraft:prismarine_crystals"],
          f"{NS}:dolphins_grace_potion")

write(assets / "lang" / "en_us.json", {
    f"item.{NS}.dolphins_grace_potion": "Potion of Dolphin's Grace",
})
# Turtle scutes and eggs are called fish scales and fish eggs: the native mod's tooltip hook names
# them (a mod can't change vanilla's own language files).


# The old water: beta-style still and flowing water, drawn here (no other mod's art). Vanilla's
# block atlas, added to here, puts them in place of vanilla's water sprites; the native mod drops
# the biome tint for one blue (`colour` in old_water.cfg). They are grey, as vanilla's are, so the
# blue comes from that tint whichever texture is drawn.
def png(path, width, height, rgba):
    """`rgba`: rows top first, four bytes a pixel."""
    rows = b"".join(b"\0" + bytes(rgba[y * width * 4:(y + 1) * width * 4]) for y in range(height))

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
                     + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b""))


def water(size, frames, flow, seed):
    """`frames` frames of `size` square, stacked: ripples that tile and loop, and for flowing water
    run one pixel down a frame. Grey (the tint makes it blue), dark to light, mostly see-through, as beta's was."""
    rnd = random.Random(seed)
    waves = [(rnd.choice([-3, -2, -1, 1, 2, 3]), rnd.choice([-3, -2, -1, 1, 2, 3]),
              rnd.choice([-2, -1, 1, 2]), rnd.uniform(0, math.tau), rnd.uniform(0.5, 1.0)) for _ in range(7)]
    deep, pale = (120, 120, 120, 168), (255, 255, 255, 210)
    total = sum(w[4] for w in waves)
    pixels = []
    for t in range(frames):
        for y in range(size):
            for x in range(size):
                v = 0.0
                for kx, ky, kt, phase, amp in waves:
                    if flow:
                        # Down the texture, `size` pixels in `frames` frames: one a frame.
                        arg = kx * x / size + ky * (y - t * size / frames) / size + phase / math.tau
                    else:
                        arg = kx * x / size + ky * y / size + kt * t / frames + phase / math.tau
                    v += amp * math.sin(math.tau * arg)
                c = max(0.0, min(1.0, (v / total + 1) / 2)) ** 2.2
                pixels.extend(round(a + (b - a) * c) for a, b in zip(deep, pale))
    return pixels


textures = assets / "textures" / "block"
png(textures / "old_water_still.png", 16, 16 * 32, water(16, 32, False, 1))
write(textures / "old_water_still.png.mcmeta", {"animation": {"frametime": 2}})
png(textures / "old_water_flow.png", 32, 32 * 32, water(32, 32, True, 2))
write(textures / "old_water_flow.png.mcmeta", {"animation": {}})
write(out / "assets" / "minecraft" / "atlases" / "blocks.json", {"sources": [
    {"type": "minecraft:single", "resource": f"{NS}:block/old_water_{texture}", "sprite": f"minecraft:block/water_{sprite}"}
    # The overlay (a side against glass or leaves) is tinted as water is, so it goes too.
    for texture, sprite in (("still", "still"), ("flow", "flow"), ("still", "overlay"))
]})

(out / "horizon-mod.toml").write_text(f'''id = "{NS}"
name = "Minecraft Misogynist Unexpansion (assets)"
version = "{VERSION}"
description = "The recipes, items and old water textures of Minecraft Misogynist Unexpansion: honey, glow ink, froglights, fish eggs and a Potion of Dolphin's Grace, crafted. Needs misogynist_unexpansion_native."
authors = ["mxpbxb"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''', encoding="utf-8")
print("built", out)
