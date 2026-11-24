"""从 msyh.ttc 子集化出 UI 专用 CJK 字体（assets/ui-cjk.ttf）。

字符集自动从 src/main.rs 的字符串字面量提取（含全部 UI 文案），
另补 ASCII 可打印字符与常用标点，防止新增文案后出现豆腐块。
新增中文文案后重跑本脚本即可。

依赖：pip install fonttools
"""
import re
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
src_text = (root / "src" / "main.rs").read_text(encoding="utf-8")

# 1) 提取源码中所有双引号字符串内的中文字符（注释不参与）
chars = set()
for lit in re.findall(r'"((?:[^"\\]|\\.)*)"', src_text):
    chars.update(c for c in lit if ord(c) > 0x7F)
# 2) ASCII 可打印字符全集（数字/字母/标点，En 模式与数字读数兜底）
chars.update(chr(c) for c in range(0x20, 0x7F))
# 3) 常用全角标点
chars.update("·×：、，。！？—…（）")
# 4) 补一个"过"以外的常见状态字兜底？——不，精确字符集 + 重跑约定
text = "".join(sorted(chars))
charset_file = root / "assets" / "charset.txt"
charset_file.write_text(text, encoding="utf-8")
print(f"charset: {len(chars)} chars")

msyh = Path("C:/Windows/Fonts/msyh.ttc")
out = root / "assets" / "ui-cjk.ttf"
cmd = [
    sys.executable, "-m", "fontTools.subset", str(msyh),
    f"--text-file={charset_file}",
    f"--output-file={out}",
    "--font-number=0",          # ttc 取 Microsoft YaHei 主 face
    "--flavor=",
    "--no-hinting",
    "--desubroutinize",
    "--layout-features=",
    "--drop-tables+=DSIG",
]
r = subprocess.run(cmd, capture_output=True, text=True)
if r.returncode != 0:
    print(r.stderr[-2000:])
    sys.exit(1)
print(f"output: {out}  size: {out.stat().st_size / 1024:.0f} KB")
