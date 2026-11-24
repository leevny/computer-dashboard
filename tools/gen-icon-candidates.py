"""生成 6 个图标候选（同一"表盘+状态灯"母题，不同风格），256px PNG + 对比拼图。
输出：assets/icon-candidates/<name>.png + compare.png
"""
import math
from PIL import Image, ImageDraw
from pathlib import Path

root = Path(__file__).resolve().parent.parent
out_dir = root / "assets" / "icon-candidates"
out_dir.mkdir(parents=True, exist_ok=True)
S = 256


def base(size=S):
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    return im, ImageDraw.Draw(im), size / 2, size / 2, size * 0.40


def gauge_ring(d, cx, cy, R, ring_color, width, start=150, end=390, dot=False):
    """表盘环（进度弧）"""
    seg = max(2, int((end - start) / 3))
    for t in range(seg):
        a0 = math.radians(start + (end - start) * t / seg)
        a1 = math.radians(start + (end - start) * (t + 1) / seg)
        r = R
        d.line([cx + r * math.cos(a0), cy + r * math.sin(a0),
                cx + r * math.cos(a1), cy + r * math.sin(a1)],
               fill=ring_color, width=width)


def ticks(d, cx, cy, R, color, n=12, w=3, r1=0.70, r2=0.84):
    for i in range(n):
        a = math.radians(i * 360 / n)
        d.line([cx + R * r1 * math.cos(a), cy + R * r1 * math.sin(a),
                cx + R * r2 * math.cos(a), cy + R * r2 * math.sin(a)],
               fill=color, width=w)


def needles(d, cx, cy, R, c1, c2, w1, w2, a1=210, a2=-30):
    d.line([cx, cy, cx + R * 0.62 * math.cos(math.radians(a1)),
            cy + R * 0.62 * math.sin(math.radians(a1))], fill=c1, width=w1)
    d.line([cx, cy, cx + R * 0.45 * math.cos(math.radians(a2)),
            cy + R * 0.45 * math.sin(math.radians(a2))], fill=c2, width=w2)
    d.ellipse([cx - w1, cy - w1, cx + w1, cy + w1], fill=(20, 24, 30, 255))


def status_dot(d, cx, cy, R, color=(230, 180, 80, 255), rr=None):
    r = rr or R * 0.12
    px, py = cx + R * 0.62, cy - R * 0.62
    d.ellipse([px - r * 1.5, py - r * 1.5, px + r * 1.5, py + r * 1.5],
              fill=(color[0], color[1], color[2], 60))
    d.ellipse([px - r, py - r, px + r, py + r], fill=color)


# ---------- 1. line-minimal 极简线框 ----------
im, d, cx, cy, R = base()
gauge_ring(d, cx, cy, R, (230, 235, 242, 200), 8)
ticks(d, cx, cy, R, (230, 235, 242, 180), 12, 2)
needles(d, cx, cy, R, (10, 151, 209, 255), (255, 255, 255, 220), 6, 4, 205, -35)
status_dot(d, cx, cy, R)
im.save(out_dir / "1-line-minimal.png")

# ---------- 2. skeuomorph 拟物仪表 ----------
im, d, cx, cy, R = base()
d.ellipse([cx - R - 10, cy - R - 10, cx + R + 10, cy + R + 10],
          fill=(28, 34, 42, 255), outline=(140, 150, 165, 255), width=10)
# 红区
gauge_ring(d, cx, cy, R * 0.93, (196, 69, 61, 255), 14, start=-15, end=45)
gauge_ring(d, cx, cy, R * 0.93, (70, 82, 98, 255), 14, start=45, end=150)
ticks(d, cx, cy, R * 0.93, (210, 218, 228, 230), 12, 4, 0.78, 0.92)
needles(d, cx, cy, R * 0.9, (230, 140, 60, 255), (240, 244, 250, 255), 9, 6, 195, -50)
d.ellipse([cx - 16, cy - 16, cx + 16, cy + 16], fill=(16, 20, 26, 255),
          outline=(150, 160, 175, 255), width=5)
status_dot(d, cx, cy, R)
im.save(out_dir / "2-skeuomorph.png")

# ---------- 3. flat 扁平化 ----------
im, d, cx, cy, R = base()
d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(10, 151, 209, 255))
d.ellipse([cx - R * 0.62, cy - R * 0.62, cx + R * 0.62, cy + R * 0.62],
          fill=(18, 22, 28, 255))
ticks(d, cx, cy, R * 0.82, (255, 255, 255, 210), 8, 6, 0.86, 1.0)
needles(d, cx, cy, R * 0.82, (255, 255, 255, 255), (230, 180, 80, 255), 10, 7, 200, -20)
d.ellipse([cx - 14, cy - 14, cx + 14, cy + 14], fill=(255, 255, 255, 255))
status_dot(d, cx, cy, R, (230, 180, 80, 255))
im.save(out_dir / "3-flat.png")

# ---------- 4. glass 玻璃质感 ----------
im, d, cx, cy, R = base()
d.rounded_rectangle([cx - R - 8, cy - R - 8, cx + R + 8, cy + R + 8],
                    radius=52, fill=(20, 26, 34, 210),
                    outline=(160, 180, 205, 150), width=4)
# 高光弧（玻璃反光）
d.arc([cx - R * 1.1, cy - R * 1.1, cx + R * 1.1, cy + R * 1.1],
      start=200, end=320, fill=(255, 255, 255, 70), width=14)
gauge_ring(d, cx, cy, R, (10, 151, 209, 160), 14)
ticks(d, cx, cy, R, (255, 255, 255, 190), 12, 3)
needles(d, cx, cy, R, (255, 255, 255, 240), (230, 180, 80, 230), 8, 6, 210, -35)
status_dot(d, cx, cy, R)
im.save(out_dir / "4-glass.png")

# ---------- 5. dark-bright 暗底亮针 ----------
im, d, cx, cy, R = base()
d.ellipse([cx - R - 6, cy - R - 6, cx + R + 6, cy + R + 6], fill=(10, 12, 16, 255),
          outline=(52, 62, 76, 255), width=5)
gauge_ring(d, cx, cy, R * 0.92, (52, 199, 89, 255), 12, start=150, end=390)
ticks(d, cx, cy, R * 0.92, (120, 130, 145, 220), 12, 3)
needles(d, cx, cy, R * 0.92, (255, 255, 255, 255), (230, 180, 80, 255), 8, 6, 210, -30)
d.ellipse([cx - 12, cy - 12, cx + 12, cy + 12], fill=(30, 36, 44, 255),
          outline=(230, 180, 80, 255), width=4)
status_dot(d, cx, cy, R)
im.save(out_dir / "5-dark-bright.png")

# ---------- 6. light-dark 亮底暗针 ----------
im, d, cx, cy, R = base()
d.ellipse([cx - R - 6, cy - R - 6, cx + R + 6, cy + R + 6], fill=(243, 246, 250, 255),
          outline=(150, 160, 175, 255), width=6)
gauge_ring(d, cx, cy, R * 0.92, (179, 64, 60, 255), 12, start=150, end=390)
ticks(d, cx, cy, R * 0.92, (90, 100, 115, 230), 12, 3)
needles(d, cx, cy, R * 0.92, (45, 55, 68, 255), (196, 69, 61, 255), 8, 6, 210, -30)
d.ellipse([cx - 12, cy - 12, cx + 12, cy + 12], fill=(255, 255, 255, 255),
          outline=(90, 100, 115, 255), width=4)
status_dot(d, cx, cy, R, (200, 150, 40, 255))
im.save(out_dir / "6-light-dark.png")

# ---------- 拼图 2×3 ----------
names = ["1-line-minimal", "2-skeuomorph", "3-flat", "4-glass", "5-dark-bright", "6-light-dark"]
labels = ["1 minimal line", "2 skeuomorph", "3 flat", "4 glass", "5 dark needle", "6 light needle"]
pad = 14
canvas = Image.new("RGBA", (S * 3 + pad * 4, S * 2 + pad * 3 + 40), (24, 28, 34, 255))
cd = ImageDraw.Draw(canvas)
for i, (n, label) in enumerate(zip(names, labels)):
    im = Image.open(out_dir / f"{n}.png")
    x = pad + (i % 3) * (S + pad)
    y = pad + (i // 3) * (S + pad + 20)
    canvas.paste(im, (x, y), im)
    cd.text((x + S // 2 - 60, y + S + 4), label, fill=(200, 210, 225, 255))
canvas.convert("RGB").save(out_dir / "compare.png")
print(f"candidates: {len(names)}  compare: {out_dir / 'compare.png'}")
