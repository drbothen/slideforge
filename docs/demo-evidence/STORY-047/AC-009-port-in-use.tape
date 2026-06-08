Output docs/demo-evidence/STORY-047/AC-009-port-in-use.gif
Output docs/demo-evidence/STORY-047/AC-009-port-in-use.webm

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

# ── Setup (hidden) ────────────────────────────────────────────────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-047"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

# ── Heading ──────────────────────────────────────────────────────────────────
Type "# STORY-047 AC-009: PortInUse -- Err(PreviewError::PortInUse) with --port hint, no panic"
Enter
Sleep 1500ms

# ── Run AC-009 port-in-use tests ─────────────────────────────────────────────
Type "cargo nextest run -p slideforge-preview -E 'test(ac009)' --no-fail-fast 2>&1"
Enter
Sleep 15s

# Hold final frame
Sleep 2s
