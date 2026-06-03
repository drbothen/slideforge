Output docs/demo-evidence/STORY-041/AC-001-009-docx-core-serialization.gif
Output docs/demo-evidence/STORY-041/AC-001-009-docx-core-serialization.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "bash"
Env PATH "/Users/jmagady/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"

# Setup — navigate to worktree, clear screen; hidden from viewer
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-041"
Enter
Sleep 300ms
Type "clear"
Enter
Sleep 300ms
Show

# Run the demo example — DocxExporter produces a .docx, all 9 ACs verified
Type "cargo run --example demo_docx -p slideforge-docx 2>/dev/null"
Enter

Sleep 10s

Sleep 2s
