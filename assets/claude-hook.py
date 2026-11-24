"""Claude Code 状态灯 hook：读 stdin JSON（含 session_id），原子写状态文件。
轻量（<1ms IO），失败静默（try/except 全包，exit 0），绝不影响 Claude Code 本身。
用法：python claude-hook.py <yellow|green|red>
"""
import json, sys, os, datetime

try:
    state = sys.argv[1] if len(sys.argv) > 1 else "yellow"
    d = json.load(sys.stdin)
    if state == "red":
        # Notification 分级：permission 类=需要接入(红)；idle 等输入类=空闲(绿，与 Stop 一致)；未知保底红
        msg = str(d.get("message", "")).lower()
        if "permission" in msg:
            state = "red"
        elif "waiting for input" in msg or "waiting for your input" in msg:
            state = "green"
        else:
            state = "red"
    sid = str(d.get("session_id", ""))[:48] or "unknown"
    p = os.path.join(os.path.expanduser("~"), ".claude", "light", "sessions")
    os.makedirs(p, exist_ok=True)
    if state == "delete":  # SessionEnd：会话结束，灯即时消失
        try:
            os.remove(os.path.join(p, sid + ".json"))
        except FileNotFoundError:
            pass
        sys.exit(0)
    tmp = os.path.join(p, ".tmp")
    json.dump({"state": state, "session": sid,
               "ts": datetime.datetime.now(datetime.timezone.utc).isoformat()},
              open(tmp, "w"), separators=(",", ":"))
    os.replace(tmp, os.path.join(p, sid + ".json"))
except Exception:
    pass
sys.exit(0)
