# STORY-021 Red Gate Log

## Story

DataSource HTTP cache + E-DAT policy error discrimination

## Red Gate Status

PASSED — test-writer Red Gate completed before implementer started. Failing tests committed on feature/S-021 before implementation began.

## Cascade Summary

- LOCAL adversary passes: 23
- Convergence: 3/3 CLEAN at passes 21, 22, 23 (BC-5.39.001 strict)
- Total findings closed: ~80
- Fix bursts: 12

## Fix Burst Commit SHAs (feature/S-021, chronological)

1. d90dee33
2. 0a5e8723
3. d5a43d84
4. d2a06c4f
5. 0bc79543
6. 1738e406
7. 39126d00
8. bf9c08d5
9. 1d135b7e
10. adf0a9df
11. bb1a2a4b
12. 37c50046
13. b1be10a1
14. 076e7c18
15. 1a33ec28
16. f5e0557f
17. 6509ca3f
18. 2b90096b
19. 2dc44fc0
20. 06245129

Total feature/S-021 commits: 22 (test-writer Red Gate + green + 20 fix burst / demo / CI commits)
Factory-artifacts commits during cascade: 5 (taxonomy v1.6 → v1.8 + story spec test table refresh)

## Key Structural Decisions During Cascade

- Pass 3: PolicyRejected variant for E-DAT-006 body-cap (distinct from SsrfBlocked / PathTraversalBlocked)
- Pass 4: Label-driven IoError vs FileNotFound routing at dispatcher boundary
- Pass 5: #[non_exhaustive] propagated to DataError + DataSourceError
- Pass 9: UTF-8 panic fix in strip_bracket_prefix (DoS vector via 16-byte slice)
- Pass 10: parse_e_dat_code added to preserve E-DAT-007..014 granular codes (silent E-DAT-003 downgrade was taxonomy violation)
- Pass 13: E-DAT-014 retired (subsumed by E-DAT-003 at dispatcher boundary)
- Pass 14: --offline hint added to HttpError + NetworkError Display per spec
- Pass 15: STRUCTURAL FIX — http.rs data_error_to_source_error decoupled from Display via canonical machine-friendly inter-layer format. Eliminated whole regression class.

## Merge

- PR: #35
- Squash commit: 362c4a1f (develop)
- Merged: 2026-05-30
- Branch deleted post-merge: feature/S-021
- Worktree removed post-merge: .worktrees/STORY-021

## Demo Evidence

Location: docs/demo-evidence/STORY-021/
Coverage: 10 ACs × 3 formats (tape/gif/webm) = 30 files + evidence-report.md
