# AC-001 — PdfExporter implements the Exporter trait; export produces valid %PDF- bytes.
# Traces to BC-4.03.002 postcondition 1.
# Tests: test_bc_4_03_002_pdf_exporter_implements_exporter_trait
#         test_bc_4_03_002_export_produces_pdf_bytes
#         test_bc_4_03_002_exporter_id_and_extension
Output docs/demo-evidence/STORY-043/AC-001-pdf-exporter-trait.gif
Output docs/demo-evidence/STORY-043/AC-001-pdf-exporter-trait.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-043"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-pdf -E 'test(exporter_implements) | test(export_produces_pdf) | test(exporter_id_and_extension)'"
Enter
Sleep 20s
Sleep 2s
