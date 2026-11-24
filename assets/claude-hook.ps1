# Claude Code status-light hook (installed by sys-gauge.exe tray toggle).
# Reads hook stdin JSON (session_id, message), atomically writes
# ~/.claude/light/sessions/<session_id>.json. Fails silently, exit 0 always.
param([string]$state = 'yellow')
try {
    $raw = [Console]::In.ReadToEnd()
    $d = $raw | ConvertFrom-Json
    $sid = if ($d.session_id) { "$($d.session_id)" } else { 'unknown' }
    if ($sid.Length -gt 48) { $sid = $sid.Substring(0, 48) }
    $dir = Join-Path $env:USERPROFILE '.claude\light\sessions'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    if ($state -eq 'delete') {
        Remove-Item (Join-Path $dir "$sid.json") -Force -ErrorAction SilentlyContinue
        exit 0
    }
    $obj = [ordered]@{
        state   = $state
        session = $sid
        ts      = (Get-Date).ToUniversalTime().ToString('o')
    }
    if ($state -eq 'red') {
        # Notification grading: permission = red, idle-waiting = green (matches Stop), unknown = red
        $msg = "$($d.message)".ToLower()
        if ($msg -like '*waiting for input*') { $obj.state = 'green' }
        else { $obj.state = 'red' }
    }
    $tmp = Join-Path $dir '.tmp'
    [IO.File]::WriteAllText($tmp, ($obj | ConvertTo-Json -Compress))
    Move-Item -Force $tmp (Join-Path $dir "$sid.json")
} catch { }
exit 0
