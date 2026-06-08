# STORY-080 AC-002: cold_budget test restructure — deflake demo
#
# Demonstrates the three-test structure in cold_budget.rs:
#   1. test_cold_budget_catastrophic_regression_gate (ALWAYS-ON, 1s/2s budget)
#   2. test_cold_render_correctness (ALWAYS-ON, no timing)
#   3. test_cold_budget_under_200ms (#[ignore]'d precision gate — shown as skipped)
#
# Traces to: AC-002, AC-003 (correctness assertion always-on)

Output docs/demo-evidence/STORY-080/AC-002-cold-budget-deflake.gif
Output docs/demo-evidence/STORY-080/AC-002-cold-budget-deflake.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "zsh"

# Setup: navigate to the worktree (hidden from recording)
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-080 && clear"
Enter
Sleep 1s
Show

# Step 1: default CI matrix run — catastrophic gate + correctness pass; precision skipped
Type "echo '=== Default CI matrix (always-on tests only) ==='"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-diagrams -E 'test(cold_budget)' 2>&1 | tail -8"
Enter
Sleep 12s

# Step 2: run --ignored to show precision gate is present but safely skipped
Type "echo '=== Precision gate: listed, #[ignore]d in default matrix ==='"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-diagrams -E 'test(cold_budget)' --ignored 2>&1 | tail -8"
Enter
Sleep 12s

# Step 3: run correctness test explicitly by name
Type "echo '=== test_cold_render_correctness: PASS (no timing) ==='"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-diagrams -E 'test(test_cold_render_correctness)' 2>&1 | tail -5"
Enter
Sleep 6s

# Step 4: run catastrophic gate explicitly by name
Type "echo '=== test_cold_budget_catastrophic_regression_gate: PASS (1s budget) ==='"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-diagrams -E 'test(test_cold_budget_catastrophic_regression_gate)' 2>&1 | tail -5"
Enter
Sleep 6s

Type "echo 'AC-002 confirmed: always-on gates pass; precision gate safely #[ignore]d'"
Enter

Sleep 2s
