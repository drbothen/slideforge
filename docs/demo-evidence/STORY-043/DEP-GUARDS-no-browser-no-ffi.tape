# DEP-GUARDS — AC-002/007/009/008: No browser/FFI/subprocess deps; krilla present.
# Traces to BC-4.03.002 invariants 1, 4 and postcondition 2.
# Tests (integration): no_forbidden_deps.rs (all 5 tests)
# Script: scripts/check-pdf-deps.sh
Output docs/demo-evidence/STORY-043/DEP-GUARDS-no-browser-no-ffi.gif
Output docs/demo-evidence/STORY-043/DEP-GUARDS-no-browser-no-ffi.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 650
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

Type "# Step 1: integration tests (Cargo.lock + Cargo.toml guards)"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-pdf --test no_forbidden_deps"
Enter
Sleep 20s

Type "# Step 2: shell CI guard (no-browser / no-FFI / no-subprocess)"
Enter
Sleep 1s
Type "bash scripts/check-pdf-deps.sh"
Enter
Sleep 8s
Sleep 2s
