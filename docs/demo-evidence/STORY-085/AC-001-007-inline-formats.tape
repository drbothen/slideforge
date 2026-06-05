Output docs/demo-evidence/STORY-085/AC-001-007-inline-formats.gif
Output docs/demo-evidence/STORY-085/AC-001-007-inline-formats.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-085"
Enter
Sleep 1s
Show

# ── Section 1: AC-001, AC-002, AC-003 — run the demo example binary ──────────
Type "# AC-001/002/003: 12 variants x OOXML/HTML/Markdown (all formats, all variants)"
Enter
Type "cargo run --example story_085_inline_formats -p slideforge-plugin-api -q 2>/dev/null"
Enter
Sleep 20s

# ── Section 2: AC-004 — module boundary fitness-function test ─────────────────
Type "# AC-004: module boundary — no private API imports (fitness-function test)"
Enter
Type "cargo nextest run -p slideforge-plugin-api -E 'test(module_boundary)' -q 2>&1 | tail -5"
Enter
Sleep 8s

# ── Section 3: AC-005 — dog-fooding grep-zero test ────────────────────────────
Type "# AC-005: grep-zero dog-fooding test (no hand-built <a:r> outside dispatch site)"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(bc_5_02_002_ac005)' -q 2>&1 | tail -5"
Enter
Sleep 8s

# ── Section 4: AC-006 — PPTX snapshot stability after refactor ───────────────
Type "# AC-006: PPTX notes XML output correctness — pinned snapshot tests"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(bc_5_02_002_ac006)' -q 2>&1 | tail -5"
Enter
Sleep 8s

# ── Section 5: AC-007 — clippy pedantic + no-unwrap gate ─────────────────────
Type "# AC-007: clippy pedantic + unwrap_used gate (no catch_unwind / unwrap on Result)"
Enter
Type "cargo clippy -p slideforge-plugin-api -p slideforge-pptx --all-targets -- -D warnings -D clippy::pedantic -D clippy::unwrap_used 2>&1 | tail -4"
Enter
Sleep 15s

# Hold final frame so the viewer can read the results
Sleep 2s
