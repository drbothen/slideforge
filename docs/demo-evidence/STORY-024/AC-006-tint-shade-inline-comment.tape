# AC-006 — lumMod/tint/shade transforms: base hex written with inline TOML comment; resolvable refs write resolved hex
Output docs/demo-evidence/STORY-024/AC-006-tint-shade-inline-comment.gif
Output docs/demo-evidence/STORY-024/AC-006-tint-shade-inline-comment.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_01_003_ec003_scheme_ref_slot_gets_inline_toml_comment) | test(test_bc_2_01_003_ec003_resolvable_scheme_ref_writes_resolved_hex) | test(test_bc_2_01_003_ec003_scheme_ref_end_to_end_round_trip)'"
Enter
Sleep 20s
Sleep 2s
