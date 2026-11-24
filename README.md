<div align="right">

[English](README.md) | [中文](README.zh-CN.md)

</div>

# sys-gauge

A 224×112 always-on-top Windows desktop dashboard: twin-needle tachometer gauges showing CPU / Memory / GPU / VRAM in real time. Built with Rust + egui — low memory footprint, low CPU usage.

## Features

- **Always-on-top frameless mini window** (224×112), draggable, quit via tray
- **Twin-needle twin gauges**: outer ring CPU / Memory, inner ring GPU / VRAM, 85–100% redline; 1 s sampling with 1 s eased needle
- **Liquid-glass visuals**: egui_bg_layer glow + per-metric ambient light; tray switches between **Liquid glass / Hybrid glass** and **Dark / Light** themes independently
- **System tray**: Claude status line, glass style & theme radio group, always-on-top toggle, quit
- **Footprint**: ~110 MB RAM, <1% of one core when idle; binary ~4.9 MB
- **Data sources**: sysinfo (CPU / memory) + Windows PDH (GPU utilization & VRAM, all vendors, no vendor SDK) + DXGI (total VRAM)

## Claude Code status lights

Vertical lights in the middle of the panel — **one light per Claude Code session**, each light disappears when its session ends (plus a 60 s staleness fallback):

| Light | Meaning |
|---|---|
| 🟢 Green | Idle / waiting for your input |
| 🟡 Yellow | Executing (breathing pulse) |
| 🔴 Red | Needs you: permission request, API error, 5-hour usage limit |
| ⚪ Off | No active session |

Install the hooks (idempotent; backs up the global `settings.json` to `.bak` on first run):

```bash
python tools/install-hooks.py
```

- Merges into the global `~/.claude/settings.json`: SessionStart=green, UserPromptSubmit / PostToolUse=yellow, Stop=green, Notification=red (graded by notification content), SessionEnd=remove light
- Data directory `~/.claude/light/`; hook scripts fail silently and never affect Claude Code itself

## Build

Windows 11 + Rust stable toolchain:

```bash
cargo build --release
```

Binary: `target/release/sys-gauge.exe` — double-click to run (GUI subsystem, no console window).


## Font subset

`assets/ui-cjk.ttf` is a ~20 KB subset generated from `msyh.ttc` by `tools/subset-font.py`
(character set auto-extracted from source strings). Regenerate it after adding new
Chinese UI strings — otherwise new glyphs render as tofu boxes.

## License

[MIT](LICENSE)
