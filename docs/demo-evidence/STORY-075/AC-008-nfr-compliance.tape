# AC-008: NFR compliance — #![forbid(unsafe_code)], #![warn(missing_docs)],
# clippy pedantic clean, = version pinning all maintained.
# Demonstrated by cargo clippy -p slideforge-brand passing with -D warnings.
# Traces to NFR-021 (pedantic), NFR-022 (clippy), NFR-023 (docs), NFR-024 (unsafe),
# NFR-025 (pinning).
Output docs/demo-evidence/STORY-075/AC-008-nfr-compliance.gif
Output docs/demo-evidence/STORY-075/AC-008-nfr-compliance.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-075"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo clippy -p slideforge-brand --all-targets --all-features -- -D warnings 2>&1 | tail -5"
Enter
Sleep 30s
Sleep 2s
