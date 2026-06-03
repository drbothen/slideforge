---
story: STORY-038
step: 3-red-gate
red_ratio: 1.0
red_count: 6
total_new_tests: 56
exempt_count: 50
denominator: 6
remediation: ""
full_exception_path: false
timestamp: 2026-06-03
stub_sha: 15df70b1
test_sha: f48db1ac
---

## Notes

50 GREEN-BY-DESIGN are PRE-EXISTING-BEHAVIOR (STORY-037 merged code already implements find_layout_index, 31-layout embedding, master/theme serialization, clrMapOvr, validate_emu). RED_RATIO 6/6 ≥ 0.5 → PASS. The 6 RED tests drive STORY-038's actual new work AND caught 2 real bugs in merged STORY-037 code: (1) duplicate <p:sldId> (4 entries for 3-slide deck), (2) master sldLayoutIdLst 32 vs 31. No UNJUSTIFIED greens.
