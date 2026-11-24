"""生成应用图标：assets/tray.ico（多尺寸 16-256）+ assets/tray.rgba（32x32 RGBA，
供托盘 include_bytes）。图标为表盘风格：深底 + 灰刻度环 + 蓝/绿双针 + 状态灯。
依赖：pip install pillow
"""
from PIL import Image, ImageDraw
import struct
from pathlib import Path

root = Path(__file__).resolve().parent.parent
SIZE = 256

im = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
d = ImageDraw.Draw(im)
cx = cy = SIZE / 2
R = SIZE * 0.42

# 深色圆底（表盘）
d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(18, 22, 28, 255),
          outline=(70, 80, 94, 255), width=SIZE // 32)
# 刻度环（灰）
import math
for i in range(24):
    a = math.radians(i * 15)
    r1, r2 = R * 0.72, R * 0.86
    d.line([cx + r1 * math.sin(a), cy - r1 * math.cos(a),
            cx + r2 * math.sin(a), cy - r2 * math.cos(a)],
           fill=(201, 210, 222, 255), width=max(2, SIZE // 64))
# 蓝色进度弧（左下段，CPU 色）
for t in range(60):
    a = math.radians(150 + t * 1.2)
    r = R * 0.79
    d.ellipse([cx + r * math.sin(a) - 3, cy - r * math.cos(a) - 3,
               cx + r * math.sin(a) + 3, cy - r * math.cos(a) + 3],
              fill=(53, 133, 201, 255))
# 绿色进度弧（右上段，内存色）
for t in range(50):
    a = math.radians(-60 + t * 1.2)
    r = R * 0.45
    d.ellipse([cx + r * math.sin(a) - 3, cy - r * math.cos(a) - 3,
               cx + r * math.sin(a) + 3, cy - r * math.cos(a) + 3],
              fill=(40, 161, 109, 255))
# 双针：蓝（CPU 指向左下）、绿（内存指向右上）
d.line([cx, cy, cx - R * 0.55, cy + R * 0.32], fill=(90, 170, 215, 255), width=max(4, SIZE // 48))
d.line([cx, cy, cx + R * 0.38, cy - R * 0.45], fill=(52, 199, 89, 255), width=max(3, SIZE // 64))
# 轴心
d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], fill=(16, 20, 26, 255),
          outline=(94, 106, 125, 255), width=3)
# 右上角状态灯（黄）
d.ellipse([cx + R * 0.55, cy - R * 0.75, cx + R * 0.75, cy - R * 0.55],
          fill=(230, 180, 80, 255))

out = root / "assets" / "tray.ico"
im.save(out, sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
print(f"ico: {out} {out.stat().st_size} bytes")

# 32x32 RGBA（托盘 tray.rgba：include_bytes 的原始 RGBA）
small = im.resize((32, 32), Image.LANCZOS)
rgba = small.tobytes()
with open(root / "assets" / "tray.rgba", "wb") as f:
    f.write(rgba)
print(f"tray.rgba: {len(rgba)} bytes")
