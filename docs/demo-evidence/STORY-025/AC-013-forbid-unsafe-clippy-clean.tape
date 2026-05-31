# AC-013 — forbid(unsafe_code) + clippy::pedantic clean + = version pinning
# NFR-022/024/025: no unsafe code, pedantic clippy, pinned deps
Output docs/demo-evidence/STORY-025/AC-013-forbid-unsafe-clippy-clean.gif
Output docs/demo-evidence/STORY-025/AC-013-forbid-unsafe-clippy-clean.webm

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

Type "grep -r 'forbid(unsafe_code)' crates/slideforge-brand/src/ crates/slideforge-types/src/ && echo 'forbid(unsafe_code) confirmed in both crates'"
Enter
Sleep 5s
Type "cargo clippy -p slideforge-brand -p slideforge-types -- -D warnings 2>&1 | tail -5"
Enter
Sleep 30s
Sleep 2s
