Output docs/demo-evidence/STORY-087/AC-001-007-014-023-024-registration.gif
Output docs/demo-evidence/STORY-087/AC-001-007-014-023-024-registration.webm

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

# ── Section 1: AC-001/007/014/023 — all three types registered, no UnknownSlideType ─
Type "# AC-001/007/014/023: status, progress_bar, weighted_composite — keyword registered + no UnknownSlideType"
Enter
Type "cargo nextest run -p slideforge-plugin-api -E 'test(~ac023_all_three) + test(~ac001_status_keyword) + test(~ac007_progress_bar_keyword) + test(~ac014_weighted_composite_keyword)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 15s

# ── Section 2: AC-024 — COLOR_CODED_TYPES and SLIDE_TYPE_KEYWORDS consistency ─
Type "# AC-024: all 4 COLOR_CODED_TYPES entries present in SLIDE_TYPE_KEYWORDS (severity_cards gap closed)"
Enter
Type "cargo nextest run -p slideforge-validate -E 'test(~ac024_all_color_coded_types_in_slide_type_keywords)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# ── Section 3: Registration count — 34 types after STORY-087 ─────────────────
Type "# Plugin registry has 34 registered slide types (31 base + status + progress_bar + weighted_composite)"
Enter
Type "cargo nextest run -p slideforge-plugin-api -E 'test(~registration_count_34_after_story_087)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# Hold final frame
Sleep 2s
