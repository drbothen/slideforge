Output docs/demo-evidence/STORY-047/AC-001-004-ws-lifecycle.gif
Output docs/demo-evidence/STORY-047/AC-001-004-ws-lifecycle.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

# ── Setup (hidden): navigate to worktree, warm the build ─────────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-047"
Enter
Sleep 500ms
Type "cargo build -p slideforge-preview -q 2>/dev/null"
Enter
Sleep 45s
Type "clear"
Enter
Sleep 300ms
Show

# ── Heading ──────────────────────────────────────────────────────────────────
Type "# STORY-047 AC-001..004: HTTP serve + WebSocket lifecycle + push_update + push_error"
Enter
Sleep 1500ms

# ── Run the WS lifecycle integration tests ───────────────────────────────────
Type "cargo nextest run -p slideforge-preview -E 'test(ac001) | test(ac002) | test(ac003) | test(ac004)' --no-fail-fast 2>&1"
Enter
Sleep 15s

# Hold final frame
Sleep 2s
