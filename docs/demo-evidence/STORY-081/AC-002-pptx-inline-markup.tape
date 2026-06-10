Output docs/demo-evidence/STORY-081/AC-002-pptx-inline-markup.gif
Output docs/demo-evidence/STORY-081/AC-002-pptx-inline-markup.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 13
Set Padding 20
Set Shell "bash"

# ── Setup (hidden): navigate to worktree, warm cargo cache ───────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-081"
Enter
Sleep 1s
Type "cargo build -p slideforge-pptx --tests 2>/dev/null"
Enter
Sleep 60s
Show

# ── AC-002: Bold → b=1 in PPTX slide XML (no literal ** in output) ───────────
Type "# AC-002: InlineNode::Bold produces b=1 attribute in PPTX a:rPr element"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(ac002_pptx_textrun_frame_bold_produces_rpr_b1)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-002: Italic → i=1 in PPTX ────────────────────────────────────────────
Type "# AC-002: InlineNode::Italic produces i=1 attribute in PPTX a:rPr element"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(ac002_pptx_italic_produces_rpr_i1)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── EC-009: Strikethrough → strike=sngStrike ─────────────────────────────────
Type "# EC-009: InlineNode::Strikethrough produces strike=sngStrike in a:rPr"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(ac002_pptx_strikethrough_produces_sng_strike)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── EC-008: Highlight → a:highlight element (not solidFill glyph recolor) ────
Type "# EC-008: InlineNode::Highlight uses a:highlight NOT solidFill (bug closed)"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(adv_p10_high_001_highlight_uses_a_highlight_not_solid_fill)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── AC-002: Code → latin typeface=Courier New ────────────────────────────────
Type "# AC-002: InlineNode::Code produces a:latin typeface=Courier New in a:rPr"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(adv_p11_med_001_body_path_code_emits_courier_new_latin)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── AC-002: Superscript → baseline=30000, Subscript → baseline=-25000 ────────
Type "# AC-002: Superscript → baseline=30000; Subscript → baseline=-25000 in a:rPr"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(body_path_superscript_emits_baseline_30000|body_path_subscript_emits_baseline_neg25000)' --no-fail-fast 2>&1 | tail -6"
Enter
Sleep 7s

# ── EC-004: Link → a:hlinkClick + External TargetMode rel ────────────────────
Type "# EC-004: InlineNode::Link produces a:hlinkClick element and External rel"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(adv_p11_med_001_body_path_link_display_text_present)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── AC-006: plain PPTX title → no b=1 (title constraint holds) ───────────────
Type "# AC-006: plain title produces no bold attribute in PPTX output"
Enter
Type "cargo nextest run -p slideforge-pptx -E 'test(ac006_pptx_plain_title_no_rpr_b1)' 2>&1 | tail -5"
Enter
Sleep 5s

# Hold final frame
Sleep 2s
