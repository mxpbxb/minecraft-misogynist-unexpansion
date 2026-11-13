"""Writes the content mod `misogynist_unexpansion`: the recipes that stand in for the removed mobs'
drops, the Potion of Dolphin's Grace, and the new names of turtle scutes and eggs.

usage: gen_content.py <content mod folder>
"""
import json
import shutil
import sys
from pathlib import Path

out = Path(sys.argv[1])
NS, VERSION = "misogynist_unexpansion", "0.1.0"
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

(out / "horizon-mod.toml").write_text(f'''id = "{NS}"
name = "Minecraft Misogynist Unexpansion (assets)"
version = "{VERSION}"
description = "The recipes and items of Minecraft Misogynist Unexpansion: honey, glow ink, froglights, fish eggs and a Potion of Dolphin's Grace, crafted. Needs misogynist_unexpansion_native."
authors = ["mxpbxb"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''', encoding="utf-8")
print("built", out)
