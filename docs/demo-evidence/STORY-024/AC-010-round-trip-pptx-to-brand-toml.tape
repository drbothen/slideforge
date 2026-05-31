# AC-010 — Round-trip: load .pptx → extract brand.toml → synthesize → BrandTemplate colors match original
Output docs/demo-evidence/STORY-024/AC-010-round-trip-pptx-to-brand-toml.gif
Output docs/demo-evidence/STORY-024/AC-010-round-trip-pptx-to-brand-toml.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back) | test(test_bc_2_01_003_effectful_extract_load_from_toml_round_trip)'"
Enter
Sleep 20s
Sleep 2s
