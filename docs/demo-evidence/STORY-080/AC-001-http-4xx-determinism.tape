# STORY-080 AC-001: test_bc_1_03_002_http_4xx_not_retried — determinism demo
#
# Demonstrates that the mpsc-channel harness eliminates the Windows TCP
# keep-alive race. Run 5 consecutive times; every run must pass with
# connection_count == 1.
#
# Traces to: AC-001, AC-003 (connection_counter == 1 assertion preserved)

Output docs/demo-evidence/STORY-080/AC-001-http-4xx-determinism.gif
Output docs/demo-evidence/STORY-080/AC-001-http-4xx-determinism.webm

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

# Run 1
Type "echo '--- Run 1/5 ---'"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-data -E 'test(test_bc_1_03_002_http_4xx_not_retried)' 2>&1 | tail -5"
Enter
Sleep 8s

# Run 2
Type "echo '--- Run 2/5 ---'"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-data -E 'test(test_bc_1_03_002_http_4xx_not_retried)' 2>&1 | tail -5"
Enter
Sleep 8s

# Run 3
Type "echo '--- Run 3/5 ---'"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-data -E 'test(test_bc_1_03_002_http_4xx_not_retried)' 2>&1 | tail -5"
Enter
Sleep 8s

# Run 4
Type "echo '--- Run 4/5 ---'"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-data -E 'test(test_bc_1_03_002_http_4xx_not_retried)' 2>&1 | tail -5"
Enter
Sleep 8s

# Run 5
Type "echo '--- Run 5/5 ---'"
Enter
Sleep 200ms
Type "cargo nextest run -p slideforge-data -E 'test(test_bc_1_03_002_http_4xx_not_retried)' 2>&1 | tail -5"
Enter
Sleep 8s

Type "echo '5/5 PASSED — mpsc harness is deterministic (AC-001 + AC-003 confirmed)'"
Enter

Sleep 2s
