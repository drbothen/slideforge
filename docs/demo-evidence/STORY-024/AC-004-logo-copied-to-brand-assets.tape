# AC-004 — Logo bytes written to brand.assets/logo.<ext>; [logo] section in brand.toml; absent logo omits section
Output docs/demo-evidence/STORY-024/AC-004-logo-copied-to-brand-assets.gif
Output docs/demo-evidence/STORY-024/AC-004-logo-copied-to-brand-assets.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_01_003_copies_logo_to_brand_assets) | test(test_bc_2_01_003_no_logo_omits_logo_section)'"
Enter
Sleep 20s
Sleep 2s
