# AC-009 — brand.toml has section headers [colors]/[fonts]/[logo]/[footer]; one field per line; stable order
Output docs/demo-evidence/STORY-024/AC-009-stable-toml-output-order.gif
Output docs/demo-evidence/STORY-024/AC-009-stable-toml-output-order.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_01_003_brand_toml_has_section_headers_and_stable_order)'"
Enter
Sleep 20s
Sleep 2s
