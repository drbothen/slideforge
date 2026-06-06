Output docs/demo-evidence/STORY-050/AC-001-003-008-full-pipeline-all-formats.gif
Output docs/demo-evidence/STORY-050/AC-001-003-008-full-pipeline-all-formats.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 13
Set Padding 20
Set Shell "bash"

# ── Setup (hidden): navigate to worktree ─────────────────────────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-050"
Enter
Sleep 1s
Show

# ── Section 1: AC-001 — PPTX pipeline ────────────────────────────────────────
Type "# AC-001/002/003/008: pipeline_pptx, pipeline_docx, pipeline_pdf, multi_format"
Enter
Type "cargo nextest run -p slideforge -E 'test(pipeline_pptx) or test(pipeline_docx) or test(pipeline_pdf) or test(multi_format)' -q 2>&1 | tail -12"
Enter
Sleep 30s

# Hold final frame
Sleep 2s
