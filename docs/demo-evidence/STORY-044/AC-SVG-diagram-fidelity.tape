# SVG / Diagram Fidelity — usvg abs_transform + fit-to-frame scaling + stroke + shear
# Traces to F-P18-001, F-P18-002, F-P19-001, F-P19-004.
# Tests: test_f_p18_001_usvg_transform_to_krilla_maps_correctly
#         test_f_p18_001_non_identity_abs_transform_reflected_in_pdf
#         test_f_p18_002_fit_to_frame_scaling_shrinks_oversized_svg
#         test_f_p19_001_stroke_only_path_emits_stroke_operator
#         test_f_p19_002_abs_transform_assertion_is_non_vacuous
#         test_f_p19_003_scaling_assertion_is_cm_matrix_not_coord_absence
#         test_f_p19_004_asymmetric_shear_maps_kx_ky_correctly
#         test_f044_004_diagram_frame_draws_svg_paths_in_export
Output docs/demo-evidence/STORY-044/AC-SVG-diagram-fidelity.gif
Output docs/demo-evidence/STORY-044/AC-SVG-diagram-fidelity.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-044"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-pdf -E 'test(f_p18) | test(f_p19) | test(diagram_frame)'"
Enter
Sleep 20s
Sleep 2s
