# AC-006 — SECTION_REGISTER_KEYS excludes "notes"; is_register_sub_block_key correctness
# Traces to BC-3.02.002 EC-004, EC-006, DIR-077-001 §5
# Demonstrates: is_register_sub_block_key("notes") == false; "report" and "detail" == true
Output docs/demo-evidence/STORY-078/AC-006-register-keys-exclude-notes.gif
Output docs/demo-evidence/STORY-078/AC-006-register-keys-exclude-notes.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-078"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-syntax -E 'test(test_register_keys_exclude_notes) | test(test_BC_3_02_002_register_key_unknown_returns_false)'"
Enter
Sleep 20s
