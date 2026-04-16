#!/usr/bin/env python3

from __future__ import annotations

from dataclasses import dataclass
import math
from pathlib import Path
import re

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parent.parent
CONTENT_PATH = ROOT / "src/content.rs"
OUTPUT_DIR = ROOT / "tmp" / "enemy-previews"
OUTPUT_PATH = OUTPUT_DIR / "contact-sheet.png"
GIF_OUTPUT_PATH = OUTPUT_DIR / "contact-sheet.gif"

PROFILE_ORDER = [
    "ScoutDrone",
    "NeedlerDrone",
    "RampartDrone",
    "SpineSentry",
    "PentaCore",
    "VoltMantis",
    "ShardWeaver",
    "PrismArray",
    "GlassBishop",
    "HexarchCore",
    "NullRaider",
    "RiftStalker",
    "SiegeSpider",
    "RiftBastion",
    "HeptarchCore",
]

PROFILE_LEVELS = {
    "ScoutDrone": 1,
    "NeedlerDrone": 1,
    "RampartDrone": 1,
    "SpineSentry": 1,
    "PentaCore": 1,
    "VoltMantis": 2,
    "ShardWeaver": 2,
    "PrismArray": 2,
    "GlassBishop": 2,
    "HexarchCore": 2,
    "NullRaider": 3,
    "RiftStalker": 3,
    "SiegeSpider": 3,
    "RiftBastion": 3,
    "HeptarchCore": 3,
}

LEVEL_PALETTES = {
    1: {
        "Base": "#8dffad",
        "DetailA": "#efff6f",
        "DetailB": "#39e8ff",
        "DetailC": "#7fb6ff",
        "DetailD": "#1fba63",
        "DetailE": "#ffe39a",
    },
    2: {
        "Base": "#c7a7ff",
        "DetailA": "#ff9df3",
        "DetailB": "#7f89ff",
        "DetailC": "#79e7ff",
        "DetailD": "#b65cff",
        "DetailE": "#ffe9b8",
    },
    3: {
        "Base": "#ffb852",
        "DetailA": "#ffe07a",
        "DetailB": "#ff6438",
        "DetailC": "#ff4f8a",
        "DetailD": "#fff27a",
        "DetailE": "#9fe7ff",
    },
}

SHEET_BG = "#050705"
HEADER_TEXT = "#c9ffd7"
MUTED_TEXT = "#8ea697"
CARD_STROKE_SOFT = (51, 255, 102, 38)

CELL_W = 184
CELL_H = 204
CELL_GAP = 18
PAGE_PAD_X = 24
PAGE_PAD_Y = 24
HEADER_H = 72
ICON_SCALE = 8
GIF_FRAME_COUNT = 18
GIF_FRAME_DURATION_MS = 70


@dataclass(frozen=True)
class LayerSpec:
    code: int
    tone_name: str
    rows: tuple[int, ...]


@dataclass(frozen=True)
class EnemySpec:
    profile_id: str
    label: str
    layers: tuple[LayerSpec, ...]


def load_font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates = []
    if bold:
        candidates.extend(
            [
                "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf",
                "/usr/share/fonts/truetype/liberation2/LiberationMono-Bold.ttf",
            ]
        )
    else:
        candidates.extend(
            [
                "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
                "/usr/share/fonts/truetype/liberation2/LiberationMono-Regular.ttf",
            ]
        )

    for candidate in candidates:
        path = Path(candidate)
        if path.exists():
            return ImageFont.truetype(str(path), size=size)
    return ImageFont.load_default()


def parse_enemy_specs() -> list[EnemySpec]:
    content = CONTENT_PATH.read_text()

    name_pattern = re.compile(r'EnemyProfileId::([A-Za-z0-9]+)\s*=>\s*"([^"]+)"')
    names: dict[str, str] = {}
    for ident, label in name_pattern.findall(content):
        names.setdefault(ident, label)

    rows_by_bits: dict[str, tuple[int, ...]] = {}
    row_pattern = re.compile(
        r"const\s+([A-Z0-9_]+)_SPRITE_BITS:\s*\[u8;\s*32\]\s*=\s*pack_enemy_sprite_rows\(\[(.*?)\]\);",
        re.S,
    )
    for bits_name, body in row_pattern.findall(content):
        rows = tuple(int(match.group(1), 2) for match in re.finditer(r"0b([01]{16})", body))
        if len(rows) != 16:
            raise ValueError(f"Expected 16 rows for {bits_name}, got {len(rows)}")
        rows_by_bits[bits_name] = rows

    layers_by_name: dict[str, LayerSpec] = {}
    layer_pattern = re.compile(
        r"const\s+([A-Z0-9_]+)_LAYER:\s*EnemySpriteLayerDef\s*=\s*EnemySpriteLayerDef\s*\{\s*"
        r"code:\s*(\d+),\s*"
        r"width:\s*ENEMY_SPRITE_WIDTH,\s*"
        r"height:\s*ENEMY_SPRITE_HEIGHT,\s*"
        r"tone:\s*EnemySpriteLayerTone::([A-Za-z]+),\s*"
        r"bits:\s*&([A-Z0-9_]+)_SPRITE_BITS,\s*"
        r"\};",
        re.S,
    )
    for layer_name, code, tone_name, bits_name in layer_pattern.findall(content):
        layers_by_name[layer_name] = LayerSpec(
            code=int(code),
            tone_name=tone_name,
            rows=rows_by_bits[bits_name],
        )

    sprite_list_pattern = re.compile(
        r"const\s+([A-Z0-9]+)_SPRITE_LAYERS:\s*&\[EnemySpriteLayerDef\]\s*=\s*&\[(.*?)\];",
        re.S,
    )
    profile_layers: dict[str, tuple[LayerSpec, ...]] = {}
    for key, body in sprite_list_pattern.findall(content):
        layer_names = re.findall(r"([A-Z0-9_]+)_LAYER", body)
        if not layer_names:
            continue
        profile_id = {
            "SCOUTDRONE": "ScoutDrone",
            "NEEDLERDRONE": "NeedlerDrone",
            "RAMPARTDRONE": "RampartDrone",
            "SPINESENTRY": "SpineSentry",
            "PENTACORE": "PentaCore",
            "VOLTMANTIS": "VoltMantis",
            "SHARDWEAVER": "ShardWeaver",
            "PRISMARRAY": "PrismArray",
            "GLASSBISHOP": "GlassBishop",
            "HEXARCHCORE": "HexarchCore",
            "NULLRAIDER": "NullRaider",
            "RIFTSTALKER": "RiftStalker",
            "SIEGESPIDER": "SiegeSpider",
            "RIFTBASTION": "RiftBastion",
            "HEPTARCHCORE": "HeptarchCore",
        }[key]
        profile_layers[profile_id] = tuple(layers_by_name[name] for name in layer_names)

    missing = [profile_id for profile_id in PROFILE_ORDER if profile_id not in profile_layers]
    if missing:
        raise ValueError(f"Missing sprite definitions for {missing}")

    specs = []
    for profile_id in PROFILE_ORDER:
        layers = profile_layers[profile_id]
        specs.append(
            EnemySpec(
                profile_id=profile_id,
                label=names.get(profile_id, profile_id),
                layers=layers,
            )
        )
    return specs


def text_size(draw: ImageDraw.ImageDraw, text: str, font: ImageFont.ImageFont) -> tuple[int, int]:
    left, top, right, bottom = draw.textbbox((0, 0), text, font=font)
    return right - left, bottom - top


def sprite_animation_state(
    code: int,
    level: int,
    tone_name: str,
    time_s: float,
) -> tuple[float, float, float, float]:
    tone_key = tone_name.lower()
    seed = code * 0.61803398875

    def wave(speed: float, phase: float = 0.0) -> float:
        return math.sin(time_s * speed + seed + phase)

    def pulse(speed: float, phase: float = 0.0) -> float:
        return wave(speed, phase) * 0.5 + 0.5

    base_bob = wave(1.1 + (code % 5) * 0.12, 0.4) * 0.006

    if tone_key == "detaila":
        state = (
            wave(2.8, 0.3) * 0.018,
            base_bob + wave(2.1, 1.2) * 0.012,
            1 + pulse(3.6, 0.9) * 0.045,
            0.76 + pulse(4.4, 0.2) * 0.24,
        )
    elif tone_key == "detailb":
        state = (
            0.0,
            base_bob * 0.6,
            1 + pulse(1.0, 1.9) * 0.028,
            0.82 + pulse(1.8, 0.6) * 0.16,
        )
    elif tone_key == "detailc":
        state = (
            wave(3.1, 0.8) * 0.014,
            base_bob + wave(4.2, 0.7) * 0.01,
            1 + pulse(2.6, 1.4) * 0.022,
            0.74 + pulse(5.1, 1.2) * 0.2,
        )
    elif tone_key == "detaild":
        state = (
            wave(2.2, 0.1) * 0.01,
            base_bob + wave(2.0, 2.1) * 0.008,
            1 + pulse(5.8, 0.4) * 0.06,
            0.6 + pulse(6.6, 0.1) * 0.4,
        )
    elif tone_key == "detaile":
        state = (
            wave(1.7, 0.9) * 0.012,
            base_bob + wave(1.4, 1.7) * 0.012,
            1 + pulse(2.1, 0.3) * 0.03,
            0.84 + pulse(2.4, 0.6) * 0.14,
        )
    else:
        state = (
            wave(1.5, 0.2) * 0.006,
            base_bob + wave(1.8, 0.5) * 0.012,
            1 + wave(1.2, 1.1) * 0.018,
            0.94 + pulse(1.1, 0.3) * 0.06,
        )

    dx, dy, scale_mul, alpha_mul = state
    if level == 2:
        return (
            dx * 1.14,
            dy * 1.1,
            1 + (scale_mul - 1) * 1.18,
            min(1.08, alpha_mul * 1.04),
        )
    if level == 3:
        surge = max(0.0, wave(6.2, 0.7)) * 0.018
        snap = 0.0 if tone_key == "base" else wave(8.0, 0.3) * 0.005
        return (
            dx * 1.32 + snap,
            dy * 1.26 - surge * (0.45 if tone_key in {"detailc", "detaild"} else 0.28),
            1 + (scale_mul - 1) * 1.45 + surge * 0.4,
            min(1.14, alpha_mul * 1.08 + surge * 0.24),
        )
    return state


def layer_image(layer: LayerSpec, scale: int, level: int) -> Image.Image:
    image = Image.new("RGBA", (16 * scale, 16 * scale), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    color = LEVEL_PALETTES[level][layer.tone_name]
    for y, row in enumerate(layer.rows):
        for x in range(16):
            if row & (1 << (15 - x)):
                draw.rectangle(
                    (
                        x * scale,
                        y * scale,
                        (x + 1) * scale - 1,
                        (y + 1) * scale - 1,
                    ),
                    fill=color,
                )
    return image


def draw_animated_icon(
    image: Image.Image,
    center_x: int,
    top_y: int,
    scale: int,
    level: int,
    layers: tuple[LayerSpec, ...],
    time_s: float,
) -> None:
    base_w = 16 * scale
    base_h = 16 * scale

    for layer in layers:
        dx_frac, dy_frac, scale_mul, alpha_mul = sprite_animation_state(
            layer.code,
            level,
            layer.tone_name,
            time_s,
        )
        base = layer_image(layer, scale, level)
        next_w = max(1, round(base_w * scale_mul))
        next_h = max(1, round(base_h * scale_mul))
        if next_w != base_w or next_h != base_h:
            base = base.resize((next_w, next_h), Image.Resampling.NEAREST)
        if alpha_mul != 1.0:
            alpha = base.getchannel("A").point(lambda value: round(value * alpha_mul))
            base.putalpha(alpha)
        x = round(center_x - base.width * 0.5 + dx_frac * base_w)
        y = round(top_y + (base_h - base.height) * 0.5 + dy_frac * base_h)
        image.alpha_composite(base, (x, y))


def render_sheet(specs: list[EnemySpec], time_s: float) -> Image.Image:
    columns = 5
    rows = (len(specs) + columns - 1) // columns
    width = PAGE_PAD_X * 2 + columns * CELL_W + (columns - 1) * CELL_GAP
    height = PAGE_PAD_Y * 2 + HEADER_H + rows * CELL_H + (rows - 1) * CELL_GAP

    image = Image.new("RGBA", (width, height), SHEET_BG)
    draw = ImageDraw.Draw(image)

    title_font = load_font(28, bold=True)
    subtitle_font = load_font(14)
    card_title_font = load_font(16, bold=True)

    draw.text((PAGE_PAD_X, PAGE_PAD_Y), "Mazocarta Enemy Previews", fill=HEADER_TEXT, font=title_font)
    draw.text(
        (PAGE_PAD_X, PAGE_PAD_Y + 40),
        "One icon per enemy, generated from layered 16x16 sprite data in src/content.rs",
        fill=MUTED_TEXT,
        font=subtitle_font,
    )

    for index, spec in enumerate(specs):
        col = index % columns
        row = index // columns
        card_x = PAGE_PAD_X + col * (CELL_W + CELL_GAP)
        card_y = PAGE_PAD_Y + HEADER_H + row * (CELL_H + CELL_GAP)
        level = PROFILE_LEVELS[spec.profile_id]

        zoom_w = 16 * ICON_SCALE
        zoom_h = 16 * ICON_SCALE
        zoom_x = card_x + (CELL_W - zoom_w) // 2
        zoom_y = card_y + 16
        draw.rounded_rectangle(
            (zoom_x - 12, zoom_y - 12, zoom_x + zoom_w + 12, zoom_y + zoom_h + 12),
            radius=8,
            fill=SHEET_BG,
            outline=CARD_STROKE_SOFT,
            width=1,
        )
        draw_animated_icon(
            image,
            zoom_x + zoom_w // 2,
            zoom_y,
            ICON_SCALE,
            level,
            spec.layers,
            time_s,
        )

        label_w, label_h = text_size(draw, spec.label, card_title_font)
        label_x = card_x + (CELL_W - label_w) // 2
        label_y = card_y + CELL_H - 34 - label_h // 2
        draw.text((label_x, label_y), spec.label, fill=HEADER_TEXT, font=card_title_font)

    return image


def main() -> None:
    specs = parse_enemy_specs()
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    frames = [
        render_sheet(specs, frame / GIF_FRAME_COUNT * (GIF_FRAME_COUNT * GIF_FRAME_DURATION_MS / 1000.0))
        for frame in range(GIF_FRAME_COUNT)
    ]
    png_frame = frames[0].convert("RGB")
    png_frame.save(OUTPUT_PATH)

    gif_frames = [frame.convert("RGB") for frame in frames]
    gif_frames[0].save(
        GIF_OUTPUT_PATH,
        save_all=True,
        append_images=gif_frames[1:],
        duration=GIF_FRAME_DURATION_MS,
        loop=0,
        disposal=2,
        optimize=False,
    )

    print(OUTPUT_PATH)
    print(GIF_OUTPUT_PATH)


if __name__ == "__main__":
    main()
