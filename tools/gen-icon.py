"""生成定稿应用图标（1 号"极简线框"候选的正式版）：
assets/tray.ico（多尺寸 16-256，小尺寸用无数字简化版保持清晰）
+ assets/tray.rgba（32x32 RGBA 简化版，托盘用）。
仪表视觉语法：240° 开口弧 + 红区段 + 单粗针 + 右上状态灯。
"""
import math
from PIL import Image, ImageDraw
from pathlib import Path

root = Path(__file__).resolve().parent.parent
START, END = 150, 390  # 240° 扫弧，底部开口

BLUE = (10, 151, 209, 255)
RED = (196, 69, 61, 230)
WHITE = (225, 232, 240, 200)
YELLOW = (230, 180, 80, 255)


def arc_points(cx, cy, r, a0, a1):
    pts = []
    n = max(2, int(abs(a1 - a0) / 3))
    for i in range(n + 1):
        a = math.radians(a0 + (a1 - a0) * i / n)
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def draw_icon(size: int, numbered: bool) -> Image.Image:
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    cx = cy = size / 2
    R = size * 0.40
    k = size / 256  # 线宽/字号缩放
    w_arc = max(3, int(14 * k))
    w_tick = max(2, int(3 * k))
    w_needle = max(4, int(11 * k))

    # 开口弧：track + 红区段 + 进度
    d.line(arc_points(cx, cy, R, START, END), fill=WHITE, width=w_arc)
    red_a0 = START + (END - START) * 0.85
    d.line(arc_points(cx, cy, R, red_a0, END), fill=RED, width=w_arc)
    prog_end = START + (END - START) * 0.72
    d.line(arc_points(cx, cy, R, START, prog_end), fill=BLUE, width=w_arc)

    # 刻度（仅大尺寸）
    if numbered:
        for i in range(12):
            a = math.radians(i * 30)
            r1, r2 = R * 0.70, R * 0.84
            d.line([cx + r1 * math.cos(a), cy + r1 * math.sin(a),
                    cx + r2 * math.cos(a), cy + r2 * math.sin(a)],
                   fill=WHITE, width=w_tick)

    # 数字 0/50/100（仅大尺寸，小尺寸下是噪点）
    if numbered:
        for v, aa in [(0, START), (50, (START + END) / 2), (100, END)]:
            a = math.radians(aa)
            px, py = cx + R * 0.58 * math.cos(a), cy + R * 0.58 * math.sin(a)
            d.text((px, py), str(v), fill=(225, 232, 240, 220), anchor="mm",
                   font_size=max(10, int(22 * k)))

    # 单粗针（72%）+ hub
    a = math.radians(START + (END - START) * 0.72)
    d.line([cx - 8 * k * math.cos(a), cy - 8 * k * math.sin(a),
            cx + R * 0.88 * math.cos(a), cy + R * 0.88 * math.sin(a)],
           fill=BLUE, width=w_needle)
    hr = max(4, int(11 * k))
    d.ellipse([cx - hr, cy - hr, cx + hr, cy + hr], fill=(16, 20, 26, 255))

    # 右上状态灯（黄，柔光）
    px, py = cx + R * 0.72, cy - R * 0.72
    pr = max(6, int(14 * k))
    d.ellipse([px - pr * 1.6, py - pr * 1.6, px + pr * 1.6, py + pr * 1.6],
              fill=(YELLOW[0], YELLOW[1], YELLOW[2], 60))
    d.ellipse([px - pr, py - pr, px + pr, py + pr], fill=YELLOW)
    return im


out_ico = root / "assets" / "tray.ico"
# 小尺寸（16/24/32/48）用无数字简化版，大尺寸（64/128/256）保留数字刻度
simple = {s: draw_icon(s, False) for s in (16, 24, 32, 48)}
full = {s: draw_icon(s, True) for s in (64, 128, 256)}
frames = {**simple, **full}
base = full[256]
base.save(out_ico, sizes=[(s, s) for s in sorted(frames)],
          append_images=[frames[s] for s in sorted(frames) if s != 256])
print(f"ico: {out_ico} {out_ico.stat().st_size} bytes")

small32 = draw_icon(32, False)
out_rgba = root / "assets" / "tray.rgba"
out_rgba.write_bytes(small32.tobytes())
print(f"tray.rgba: {out_rgba.stat().st_size} bytes")

preview = Image.new("RGBA", (256 * 2 + 20, 256), (40, 44, 52, 255))
preview.paste(draw_icon(256, False), (0, 0), draw_icon(256, False))
preview.paste(full[256], (256 + 20, 0), full[256])
preview.convert("RGB").save(root / "assets" / "icon-final-compare.png")
print("compare: assets/icon-final-compare.png (left: tray/small style, right: full 256)")
