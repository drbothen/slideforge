# STORY-037: PPTX Core Serialization — per-AC demo recording
# Covers AC-001..AC-010 in a single terminal run of the demo_pptx example.
#
# AC-001: Exporter trait (id/extension)
# AC-002: ZIP part list (all required parts)
# AC-003: [Content_Types].xml completeness
# AC-004: Placeholder idx=0 in slide XML
# AC-005: Integer EMU coordinates
# AC-006: Element ordering (nvSpPr < spPr < txBody)
# AC-007: Determinism (SHA-256 match)
# AC-008: LibreOffice CI gate (noted as STORY-052)
# AC-009: Relationship chain completeness
# AC-010: Report/detail register content absent from slide bodies

Output AC-001-010-pptx-core-serialization.gif
Output AC-001-010-pptx-core-serialization.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 50ms

# Setup (hidden from viewer)
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-037"
Enter
Sleep 2s
Show

# Title frame
Type "# STORY-037: PPTX Core Serialization — demo_pptx example"
Enter
Sleep 800ms

Type "cargo run --example demo_pptx -p slideforge-pptx 2>&1 | head -80"
Enter
Sleep 30s

Sleep 2000ms
