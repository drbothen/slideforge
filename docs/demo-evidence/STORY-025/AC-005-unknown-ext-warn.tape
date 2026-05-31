# AC-005 (error path) — unknown logo extension emits warn, still resolves
# F-025-002: unrecognized extension -> application/octet-stream + tracing::warn
# Not a hard error at overlay-resolution layer (parser rejection deferred to STORY-008/009)
Output docs/demo-evidence/STORY-025/AC-005-unknown-ext-warn.gif
Output docs/demo-evidence/STORY-025/AC-005-unknown-ext-warn.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-025"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-brand -E 'test(test_f025_002_unknown_ext_overlay_resolves_with_warn) | test(test_f025_003)'"
Enter
Sleep 30s
Sleep 2s
