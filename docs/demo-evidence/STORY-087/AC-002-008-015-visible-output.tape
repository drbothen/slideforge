Output docs/demo-evidence/STORY-087/AC-002-008-015-visible-output.gif
Output docs/demo-evidence/STORY-087/AC-002-008-015-visible-output.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-087"
Enter
Sleep 1s
Show

# ── Section 1: AC-002 — status label "On Track" visible in LaidOutDeck ───────
Type "# AC-002: status slide — label 'On Track' visible in LaidOutDeck (FrameContent::Body via Stage-2b threading)"
Enter
Type "cargo nextest run -p slideforge -E 'test(~AC_002_status_label_visible_in_output)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# ── Section 2: AC-008 — progress_bar label + ColorBar frame visible ──────────
Type "# AC-008: progress_bar — label '75% complete' visible + FrameContent::ColorBar with filled_width_emu > 0"
Enter
Type "cargo nextest run -p slideforge -E 'test(~AC_008_progress_bar_label_and_bar_visible)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# ── Section 3: AC-008 — ColorBar renders as solid-fill <p:sp> in PPTX XML ───
Type "# AC-008 (PPTX export): ColorBar renders as solid-fill <p:sp> element in PPTX slide XML"
Enter
Type "cargo nextest run -p slideforge -E 'test(~AC_008_progress_bar_bar_rendered_in_pptx_xml)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 15s

# ── Section 4: AC-015 — weighted_composite aggregate + component labels visible ─
Type "# AC-015: weighted_composite — aggregate label + component labels visible in LaidOutDeck"
Enter
Type "cargo nextest run -p slideforge -E 'test(~AC_015_weighted_composite_labels_visible)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# Hold final frame
Sleep 2s
