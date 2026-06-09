Output docs/demo-evidence/STORY-088/AC-007-e2e-pptx-runs.gif
Output docs/demo-evidence/STORY-088/AC-007-e2e-pptx-runs.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-088"
Enter
Sleep 1s
Show

# ── Show the fixture: direct list-literal bullets DSL source ─────────────────
Type '# AC-007: fixture -- bullets: ["Item A","Item B","Item C"] direct list-literal'
Enter
Type "cat crates/slideforge/tests/fixtures/story-088-bullets-direct-literal.sf"
Enter
Sleep 2s

# ── AC-007 E2E test: build() -> PPTX -> asserts >= 3 <a:r> text runs ──────────
Type '# AC-007 E2E: bullets: [...] -> PPTX containing >=3 <a:r> runs with bullet text'
Enter
Type "cargo nextest run -p slideforge -E 'test(ac007_direct_list_literal_bullets)' 2>&1 | tail -8"
Enter
Sleep 35s

# Hold final frame
Sleep 2s
