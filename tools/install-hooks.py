"""安装 Claude Code 状态灯 hooks：合并全局 settings.json（先备份 .bak，只增不改），复制 hook 脚本。
幂等：重复运行不产生重复 hook 条目。
"""
import json, os, shutil, sys

home = os.path.expanduser("~")
settings_path = os.path.join(home, ".claude", "settings.json")
bak_path = settings_path + ".bak"
hook_dst = os.path.join(home, ".claude", "light", "claude-hook.py")

# 1) 复制 hook 脚本到 ~/.claude/light/
os.makedirs(os.path.dirname(hook_dst), exist_ok=True)
shutil.copy2(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "claude-hook.py"),
             hook_dst)

# 2) 备份 settings.json（首次；已存在 bak 不覆盖，保住最早原始状态）
if os.path.exists(settings_path) and not os.path.exists(bak_path):
    shutil.copy2(settings_path, bak_path)
    print("settings.json -> settings.json.bak")

# 3) 合并 hooks
settings = {}
if os.path.exists(settings_path):
    with open(settings_path, encoding="utf-8") as f:
        settings = json.load(f)

py = sys.executable.replace("\\", "/")
events = {"SessionStart": "green", "UserPromptSubmit": "yellow", "Stop": "green",
          "Notification": "red", "SessionEnd": "delete"}
hooks = settings.setdefault("hooks", {})
added = 0
for event, state in events.items():
    cmd = f'{py} "{hook_dst}" {state}'
    lst = hooks.setdefault(event, [])
    # 先移除本脚本管理的旧条目（参数映射修正时防止新旧残留），再插入正确条目
    for h in lst[:]:
        if isinstance(h, dict) and any(
                "claude-hook.py" in hh.get("command", "")
                for hh in h.get("hooks", []) if isinstance(hh, dict)):
            lst.remove(h)
    lst.append({"hooks": [{"type": "command", "command": cmd, "timeout": 10}]})
    added += 1

# 4) 原子写
tmp = settings_path + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(settings, f, indent=2, ensure_ascii=False)
os.replace(tmp, settings_path)
print(f"hooks merged: +{added} (SessionStart/UserPromptSubmit=yellow, Stop=green, Notification=red, SessionEnd=delete)")
