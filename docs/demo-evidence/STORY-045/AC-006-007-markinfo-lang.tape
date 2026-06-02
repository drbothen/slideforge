# STORY-045 AC-006 + AC-007: /MarkInfo << /Marked true >> and /Lang from deck metadata
# AC-006: MarkInfo /Marked true present in document catalog
# AC-007: /Lang entry set from deck.metadata.lang (en-US, de, multilingual)
#
# Evidence: run AC-006 and AC-007 tests from tests/pdf_ua1.rs

Output docs/demo-evidence/STORY-045/AC-006-007-markinfo-lang.gif
Output docs/demo-evidence/STORY-045/AC-006-007-markinfo-lang.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-045"
Enter
Wait+Line /\$/
Type "clear"
Enter
Wait+Line /\$/
Show

Type "cargo nextest run -p slideforge-pdf --test pdf_ua1 -E 'test(mark_info)|test(lang_set)|test(lang_de)|test(lang_none)|test(ec004_doc_level_lang)' --no-fail-fast"
Enter
Wait+Line /Summary/
Sleep 2s
