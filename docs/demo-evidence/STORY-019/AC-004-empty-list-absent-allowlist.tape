# AC-004 — Empty list blocks all; absent allowlist permits all
# Verifies: empty domains list = deny-all; no allowlist config = permit-all
Output docs/demo-evidence/STORY-019/AC-004-empty-list-absent-allowlist.gif
Output docs/demo-evidence/STORY-019/AC-004-empty-list-absent-allowlist.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-019"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-004: Empty list blocks all / absent allowlist permits all
Type "cargo nextest run -p slideforge-data -E 'test(empty_list_blocks_all) | test(absent_allowlist_permits)'"
Enter
Wait+Screen /Summary/
Sleep 2s
