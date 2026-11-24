"""图标候选 v2：仪表视觉语法（开口弧 240° + 红区段 + 单粗针 + 数字刻度 + 右上黄灯）。
6 种风格：线框 / 拟物 / 扁平 / 玻璃 / 暗底亮针 / 产品微缩。
输出：assets/icon-candidates-v2/<name>.png + compare.png
"""
import math
from PIL import Image, ImageDraw
from pathlib import Path

root = Path(__file__).resolve().parent.parent
out_dir = root / "assets" / "icon-candidates-v2"
out_dir.mkdir(parents=True, exist_ok=True)
S = 256

START, END = 150, 390  # 240° 扫弧，底部开口（0°=右侧，顺时针）


def arc_points(cx, cy, r, a0, a1, seg=None):
    seg = seg or max(2, int(abs(a1 - a0) / 3))
    pts = []
    for i in range(seg + 1):
        a = math.radians(a0 + (a1 - a0) * i / seg)
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def draw_gauge_base(d, cx, cy, R, track_c, prog_c, red_c, prog_pct=72, red_start=85,
                    w_track=12, w_prog=14, num_c=(230, 235, 242, 220), num_r=0.62,
                    num_size=None, hub_c=(16, 20, 26, 255), hub_outline=None):
    """仪表语法：开口弧（track+红区+进度）+ 数字 0/50/100 + 单粗针 + 中心 hub"""
    red_a0 = START + (END - START) * red_start / 100
    # track（全程）
    d.line(arc_points(cx, cy, R, START, END), fill=track_c, width=w_track)
    # 红区（85–100%）
    d.line(arc_points(cx, cy, R, red_a0, END), fill=red_c, width=w_prog)
    # 进度弧（0–prog%，红区前）
    prog_end = START + (END - START) * min(prog_pct, red_start) / 100
    d.line(arc_points(cx, cy, R, START, prog_end), fill=prog_c, width=w_prog)
    # 数字 0 / 50 / 100（弧内侧）
    ns = num_size or max(14, int(R * 0.13))
    for v, aa in [(0, START), (50, (START + END) / 2), (100, END)]:
        a = math.radians(aa)
        px, py = cx + R * num_r * math.cos(a), cy + R * num_r * math.sin(a)
        d.text((px, py), str(v), fill=num_c, anchor="mm", font_size=ns)


def draw_needle(d, cx, cy, R, pct, color, width, hub_r=None, hub_c=(16, 20, 26, 255),
                hub_outline=None):
    a = math.radians(START + (END - START) * min(pct, 100) / 100)
    tip_r = R * 0.88
    d.line([cx - 8 * math.cos(a), cy - 8 * math.sin(a),
            cx + tip_r * math.cos(a), cy + tip_r * math.sin(a)], fill=color, width=width)
    hr = hub_r or width
    d.ellipse([cx - hr, cy - hr, cx + hr, cy + hr], fill=hub_c,
              outline=hub_outline, width=3 if hub_outline else 0)


def status_dot(d, cx, cy, R, color=(230, 180, 80, 255)):
    px, py = cx + R * 0.72, cy - R * 0.72
    r = R * 0.10
    d.ellipse([px - r * 1.6, py - r * 1.6, px + r * 1.6, py + r * 1.6],
              fill=(color[0], color[1], color[2], 70))
    d.ellipse([px - r, py - r, px + r, py + r], fill=color)


# ---------- 1. line-minimal 线框（透明底、细白开口弧） ----------
im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(im)
cx = cy = S / 2
R = S * 0.40
draw_gauge_base(d, cx, cy, R, (225, 232, 240, 190), (10, 151, 209, 255),
                (196, 69, 61, 230), 72, w_track=9, w_prog=11, num_r=0.58, num_size=22)
draw_needle(d, cx, cy, R, 72, (10, 151, 209, 255), 9)
status_dot(d, cx, cy, R)
im.save(out_dir / "1-line-minimal.png")

# ---------- 2. skeuomorph 拟物转速表 ----------
im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(im)
cx = cy = S / 2
R = S * 0.38
d.ellipse([cx - R - 16, cy - R - 16, cx + R + 16, cy + R + 16], fill=(26, 32, 40, 255),
          outline=(150, 160, 175, 255), width=8)
draw_gauge_base(d, cx, cy, R, (62, 72, 86, 255), (224, 138, 48, 255),
                (196, 69, 61, 255), 68, w_track=13, w_prog=15, num_r=0.60, num_size=20)
draw_needle(d, cx, cy, R, 68, (230, 140, 60, 255), 11, hub_r=14,
            hub_c=(16, 20, 26, 255), hub_outline=(160, 170, 185, 255))
status_dot(d, cx, cy, R)
im.save(out_dir / "2-skeuomorph.png")

# ---------- 3. flat 扁平 ----------
im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(im)
cx = cy = S / 2
R = S * 0.44
d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(10, 151, 209, 255))
draw_gauge_base(d, cx, cy, R * 0.92, (255, 255, 255, 90), (255, 255, 255, 255),
                (255, 90, 90, 255), 70, w_track=13, w_prog=15,
                num_c=(255, 255, 255, 235), num_r=0.60, num_size=22)
draw_needle(d, cx, cy, R * 0.92, 70, (255, 255, 255, 255), 11, hub_r=13,
            hub_c=(18, 22, 28, 255))
status_dot(d, cx, cy, R, (255, 214, 90, 255))
im.save(out_dir / "3-flat.png")

# ---------- 4. glass 玻璃（圆角方底，产品同款） ----------
im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(im)
d.rounded_rectangle([12, 12, S - 12, S - 12], radius=56, fill=(20, 26, 34, 215),
                    outline=(160, 180, 205, 140), width=4)
cx = cy = S / 2
R = S * 0.34
d.arc([cx - R * 1.25, cy - R * 1.25, cx + R * 1.25, cy + R * 1.25],
      start=210, end=320, fill=(255, 255, 255, 60), width=12)
draw_gauge_base(d, cx, cy, R, (10, 151, 209, 150), (255, 255, 255, 240),
                (229, 84, 75, 230), 74, w_track=12, w_prog=13,
                num_c=(255, 255, 255, 220), num_r=0.60, num_size=20)
draw_needle(d, cx, cy, R, 74, (255, 255, 255, 245), 9)
status_dot(d, cx, cy, R)
im.save(out_dir / "4-glass.png")

# ---------- 5. dark-bright 暗底亮针 ----------
im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(im)
cx = cy = S / 2
R = S * 0.40
d.ellipse([cx - R - 10, cy - R - 10, cx + R + 10, cy + R + 10], fill=(10, 12, 16, 255),
          outline=(52, 62, 76, 255), width=5)
draw_gauge_base(d, cx, cy, R, (46, 54, 66, 255), (52, 199, 89, 255),
                (229, 84, 75, 235), 76, w_track=12, w_prog=14, num_r=0.60, num_size=20)
draw_needle(d, cx, cy, R, 76, (255, 255, 255, 255), 10, hub_r=12)
status_dot(d, cx, cy, R)
im.save(out_dir / "5-dark-bright.png")

# ---------- 6. product-micro 产品微缩（双迷你盘 + 中间黄灯 + 底部读数条） ----------
im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(im)
d.rounded_rectangle([8, 8, S - 8, S - 8], radius=44, fill=(18, 22, 28, 230),
                    outline=(70, 80, 94, 200), width=4)
# 左迷你盘（CPU，蓝，开口弧+单针）
for cx, cy, R, arc_c, needle_c in [(S * 0.27, S * 0.42, S * 0.155, (53, 133, 201, 255), (90, 170, 215, 255)),
                                   (S * 0.73, S * 0.42, S * 0.155, (40, 161, 109, 255), (60, 190, 130, 255))]:
    d.line(arc_points(cx, cy, R, START, END), fill=(70, 80, 94, 255), width=7)
    d.line(arc_points(cx, cy, R, START, START + (END - START) * 0.66),
           fill=arc_c, width=7)
    a = math.radians(START + (END - START) * 0.66)
    d.line([cx, cy, cx + R * 0.8 * math.cos(a), cy + R * 0.8 * math.sin(a)],
           fill=needle_c, width=5)
    d.ellipse([cx - 7, cy - 7, cx + 7, cy + 7], fill=(16, 20, 26, 255))
# 中间竖排状态灯（黄，产品同款）
d.ellipse([S / 2 - 11, S * 0.30 - 11, S / 2 + 11, S * 0.30 + 11],
          fill=(230, 180, 80, 255))
# 底部读数条（两个色块，模拟指标标注）
d.rounded_rectangle([S * 0.10, S * 0.74, S * 0.44, S * 0.90], radius=10,
                    fill=(53, 133, 201, 60))
d.rounded_rectangle([S * 0.56, S * 0.74, S * 0.90, S * 0.90], radius=10,
                    fill=(40, 161, 109, 60))
d.text((S * 0.27, S * 0.82), "12%", fill=(90, 170, 215, 255), anchor="mm", font_size=26)
d.text((S * 0.73, S * 0.82), "76%", fill=(60, 190, 130, 255), anchor="mm", font_size=26)
im.save(out_dir / "6-product-micro.png")

# ---------- 拼图 2×3 ----------
names = ["1-line-minimal", "2-skeuomorph", "3-flat", "4-glass", "5-dark-bright", "6-product-micro"]
labels = ["1 minimal line", "2 skeuomorph", "3 flat", "4 glass", "5 dark bright", "6 product micro"]
pad = 14
canvas = Image.new("RGBA", (S * 3 + pad * 4, S * 2 + pad * 3 + 40), (24, 28, 34, 255))
cd = ImageDraw.Draw(canvas)
for i, (n, label) in enumerate(zip(names, labels)):
    im = Image.open(out_dir / f"{n}.png")
    x = pad + (i % 3) * (S + pad)
    y = pad + (i // 3) * (S + pad + 20)
    canvas.paste(im, (x, y), im)
    cd.text((x + S // 2 - 50, y + S + 4), label, fill=(200, 210, 225, 255))
canvas.convert("RGB").save(out_dir / "compare.png")
print(f"candidates v2: {len(names)}  compare: {out_dir / 'compare.png'}")
