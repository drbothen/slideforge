# STORY-038 Demo — PPTX Layout Compliance
# Demonstrates all AC-001..AC-012 + S1/S3 via the demo_layout example binary.
# Pre-built with: cargo build --example demo_layout -p slideforge-pptx
#
# Font: FiraCode Nerd Font Mono (confirmed available via fc-list)

Output /Users/jmagady/Dev/slideforge/.worktrees/STORY-038/docs/demo-evidence/STORY-038/AC-001-012-layout-compliance.gif
Output /Users/jmagady/Dev/slideforge/.worktrees/STORY-038/docs/demo-evidence/STORY-038/AC-001-012-layout-compliance.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1280
Set Height 720
Set FontSize 13
Set Padding 20
Set TypingSpeed 40ms

# Hide setup — change to worktree root
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-038"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 200ms
Show

# Run the pre-built demo binary directly (cargo run would trigger recompile output)
Type "./target/debug/examples/demo_layout 2>/dev/null"
Sleep 200ms
Enter

# Allow 12 seconds for all output lines to scroll through
Sleep 12s
