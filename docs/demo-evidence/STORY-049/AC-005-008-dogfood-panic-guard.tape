Output docs/demo-evidence/STORY-049/AC-005-008-dogfood-panic-guard.gif
Output docs/demo-evidence/STORY-049/AC-005-008-dogfood-panic-guard.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-049"
Enter
Sleep 1s
Show

# ── Section 1: AC-005 — cargo tree shows pptx excludes pdf/html ──────────────
Type "# AC-005: cargo tree -p slideforge-pptx excludes slideforge-pdf and slideforge-html"
Enter
Type "cargo tree -p slideforge-pptx 2>&1 | grep -E 'slideforge-pdf|slideforge-html' && echo FOUND || echo 'CLEAN: no pdf/html cross-dep in pptx'"
Enter
Sleep 8s

# ── Section 2: AC-006 — clippy zero visibility violations ────────────────────
Type "# AC-006: cargo clippy --workspace zero visibility-violation warnings"
Enter
Type "cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -4"
Enter
Sleep 30s

# ── Section 3: AC-007 — external dog-fooding plugin test ─────────────────────
Type "# AC-007: external TestDataSource (plugin-api only) compiles + registers"
Enter
Type "cargo nextest run --test external_plugin_test -q 2>&1 | tail -6"
Enter
Sleep 10s

# ── Section 4: AC-008 — panic dispatch test (PluginPanic, no crash) ──────────
Type "# AC-008: panicking plugin → Err(PluginError::PluginPanic), no process crash"
Enter
Type "cargo nextest run -p slideforge -E 'test(panic)' -q 2>&1 | tail -8"
Enter
Sleep 10s

# ── Section 5: BONUS — check-panic-profile.sh CI guard ───────────────────────
Type "# BONUS: check-panic-profile.sh — panic=unwind safety perimeter"
Enter
Type "bash scripts/check-panic-profile.sh"
Enter
Sleep 8s

# Hold final frame
Sleep 2s
