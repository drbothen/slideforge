# AC-011 — forbid(unsafe_code) present; clippy::pedantic clean; = version pinning in Cargo.toml
Output docs/demo-evidence/STORY-024/AC-011-forbid-unsafe-clippy-pinning.gif
Output docs/demo-evidence/STORY-024/AC-011-forbid-unsafe-clippy-pinning.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-024"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "grep -n 'forbid(unsafe_code)' crates/slideforge-brand/src/lib.rs"
Enter
Sleep 3s
Type "cargo clippy -p slideforge-brand --all-targets --all-features -- -D warnings 2>&1 | tail -5"
Enter
Sleep 30s
Sleep 2s
