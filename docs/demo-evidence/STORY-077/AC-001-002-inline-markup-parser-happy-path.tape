# VHS Demo — STORY-077 AC-001 / AC-002
# Inline markup parser: section detail/report fields produce
# structural FieldValue::Inlines (Bold, Italic, Code, Link, Xref, etc.)
# instead of flat Literal("**text**") strings.
#
# AC-001: SectionBlock.body["detail"] holds FieldValue::Inlines (not Value::Str)
# AC-002: Parser recognizes **, _, `, [text](url), ^super^, ~sub~, ~~strike~~, ==highlight==
#
# Modality: cargo nextest (CLI binary STORY-055 not yet built)
# Commit: f7e4d444

Output .factory/demos/STORY-077/AC-001-002-inline-markup-parser-happy-path.gif
Output .factory/demos/STORY-077/AC-001-002-inline-markup-parser-happy-path.webm

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
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-077"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# Show the test target we are running
Type "# AC-001/002: inline markup parser — bold, italic, code, link, xref all produce structural nodes"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-syntax -E 'test(template_inline_markup_tests)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 30s
