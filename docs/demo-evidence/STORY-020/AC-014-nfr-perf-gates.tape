# AC-014 — NFR-036/037 perf gates: xlsx_10k_rows < 2000ms; sqlite_10k_rows < 500ms
Output docs/demo-evidence/STORY-020/AC-014-nfr-perf-gates.gif
Output docs/demo-evidence/STORY-020/AC-014-nfr-perf-gates.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-020"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo test --test perf_smoke -- --ignored --nocapture 2>&1 | tail -10"
Enter
Sleep 8s
Sleep 2s
