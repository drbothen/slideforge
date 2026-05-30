# AC-010 — #![forbid(unsafe_code)] + clippy::pedantic clean + = version pinning
Output docs/demo-evidence/STORY-021/AC-010-forbid-unsafe-clippy-pinning.gif
Output docs/demo-evidence/STORY-021/AC-010-forbid-unsafe-clippy-pinning.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-021"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "# AC-010: forbid(unsafe_code) + clippy pedantic clean + = version pinning"
Enter
Sleep 1s
Type "grep 'forbid(unsafe_code)' crates/slideforge-data/src/lib.rs"
Enter
Sleep 2s
Type "cargo clippy -p slideforge-data --all-targets --all-features -- -D warnings 2>&1 | tail -3"
Enter
Sleep 15s
Sleep 2s
