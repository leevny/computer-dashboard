<div align="right">

[English](README.md) | [中文](README.zh-CN.md)

</div>

# gaugelet

224×112 的 Windows 桌面常驻仪表盘：Rust + egui 实现，双针双盘（汽车转速表风格）实时显示 CPU / 内存 / GPU / 显存占用，低内存低 CPU 占用，默认置顶。

## 功能特性

- **置顶无边框小窗**（224×112），整窗可拖动移动，托盘退出
- **双针双盘**：外环 CPU / 内存，内环 GPU / 显存，85–100% 红区；1 秒采样，指针 1s 缓动
- **液态玻璃视觉**：egui_bg_layer 光晕 + 指标色氛围，托盘切「液态玻璃 / 混合玻璃」，深色 / 浅色主题独立切换
- **系统托盘**：Claude 状态行、玻璃风格与主题单选、窗口置顶、退出
- **资源占用**：内存 ~110 MB、CPU <1% 单核（空闲时更低）；exe ~4.9 MB
- **数据源**：sysinfo（CPU / 内存）+ Windows PDH（GPU 利用率与显存，全厂商通用，无需厂商 SDK）+ DXGI（显存总量）

## Claude Code 状态灯

面板中缝竖排多灯，**一灯一个 Claude Code 会话**，会话结束对应灯即时消失（另有 60s 无更新自动消失的兜底）：

| 灯色 | 含义 |
|---|---|
| 🟢 绿 | 空闲 / 等你输入 |
| 🟡 黄 | 执行中（呼吸闪烁） |
| 🔴 红 | 需要接入：权限请求、API 报错、5 小时限额 |
| ⚪ 无灯 | 无活跃会话 |

安装 hooks（幂等，首次运行自动备份全局 `settings.json` 为 `.bak`）：

```bash
python tools/install-hooks.py
```

- 写入全局 `~/.claude/settings.json`：SessionStart=绿、UserPromptSubmit / PostToolUse=黄、Stop=绿、Notification=红（按通知内容分级，权限请求红、空闲等输入绿）、SessionEnd=删灯
- 数据目录 `~/.claude/light/`；hook 脚本失败静默退出，不影响 Claude Code 本身
- 界面语言跟随系统（中文 / 英文自动检测）

## 构建

Win11 + Rust stable toolchain：

```bash
cargo build --release
```

产物 `target/release/gaugelet.exe`，双击运行（GUI subsystem，无控制台窗口）。


## 字体子集

`assets/ui-cjk.ttf` 是由 `tools/subset-font.py` 从 `msyh.ttc` 子集化的 ~20 KB 字体
（字符集自动从源码字符串提取）。新增中文 UI 文案后请重跑该脚本重新生成，否则新字形会显示为豆腐块。

## License

[MIT](LICENSE)
