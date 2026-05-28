# AC-014 — Crate-level lint attributes verified via clippy
# Verifies: #![deny(unsafe_code)], clippy::pedantic, #![warn(missing_docs)] pass
Output docs/demo-evidence/STORY-019/AC-014-crate-lint-attrs.gif
Output docs/demo-evidence/STORY-019/AC-014-crate-lint-attrs.webm

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

# AC-014: Crate-level lint attributes — clippy clean
Type "cargo clippy -p slideforge-data --all-targets --all-features -- -D warnings"
Enter
Wait+Screen /Finished/
Sleep 2s
