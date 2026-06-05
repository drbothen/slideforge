Output docs/demo-evidence/STORY-049/AC-001-004-registry-assembly.gif
Output docs/demo-evidence/STORY-049/AC-001-004-registry-assembly.webm

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

# ── Section 1: AC-001 + AC-004 — surface_count==10 and canonical names ───────
Type "# AC-001/004: register_bundled_plugins → surface_count==10, canonical names"
Enter
Type "cargo nextest run -p slideforge -E 'test(register_bundled) or test(surface_count) or test(surface_names) or test(default_registry)' -q 2>&1 | tail -8"
Enter
Sleep 15s

# ── Section 2: AC-002 — empty builder returns Err(MissingSurface) not panic ──
Type "# AC-002: empty PluginRegistryBuilder::default().build() → Err(MissingSurface)"
Enter
Type "cargo nextest run -p slideforge -E 'test(empty_builder) or test(partial_builder)' -q 2>&1 | tail -6"
Enter
Sleep 10s

# ── Section 3: AC-003 — cargo build --workspace passes ───────────────────────
Type "# AC-003: cargo build --workspace — all 10 surfaces compile"
Enter
Type "cargo build --workspace 2>&1 | tail -5"
Enter
Sleep 15s

# Hold final frame
Sleep 2s
