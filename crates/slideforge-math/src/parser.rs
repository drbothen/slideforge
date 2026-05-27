//! LaTeX → [`MathAst`] parser.
//!
//! Converts the raw LaTeX source string from a [`slideforge_types::MathNode`]
//! into a structured [`MathAst`] for downstream rendering.
//!
//! ## Error accumulation
//!
//! The parser accumulates ALL errors in one pass (BC-5.29.002). It never stops
//! on the first unsupported command. The returned
//! `(Option<MathAst>, Vec<MathDiagnostic>)` pair lets callers decide whether to
//! proceed with a partial AST or surface all diagnostics.
//!
//! ## Supported LaTeX subset (v1.0)
//!
//! Only the commands in the table below are recognized. Everything else produces
//! a [`MathRendererError::UnsupportedCommand`] diagnostic with a correction hint.
//!
//! | Category | Commands |
//! |----------|---------|
//! | Structure | `^`, `_`, `\frac`, `\sqrt`, `{`, `}` |
//! | Accents | `\hat`, `\bar`, `\tilde`, `\vec`, `\dot`, `\ddot` |
//! | Greek | `\alpha` … `\omega`, `\Alpha` … `\Omega` |
//! | Operators | `\sum`, `\prod`, `\int`, `\lim`, `\max`, `\min` |
//! | Symbols | `\cdot`, `\times`, `\div`, `\infty`, `\pm`, `\mp`, `\leq`, `\geq` |
//! | Spaces | `\,`, `\;`, `\quad`, `\qquad` |

use std::sync::Arc;

use slideforge_types::SourceSpan;

use crate::ast::{AccentKind, MathAst, MathMode, MathNode};
use crate::error::{MathDiagnostic, MathRendererError};

/// Maximum brace-nesting depth allowed by the parser.
///
/// When a group `{...}` would exceed this depth, a [`MathRendererError::ParseError`]
/// diagnostic is emitted and the group is treated as empty to prevent runaway
/// recursion or stack exhaustion.
pub const MAX_DEPTH: usize = 256;

/// Parse a LaTeX source string into a [`MathAst`].
///
/// # Arguments
///
/// * `latex` — Raw LaTeX source (without surrounding `$` delimiters).
/// * `mode`  — Whether the expression is inline or display-mode.
/// * `span`  — Source location of the entire math expression (for diagnostics).
///
/// # Returns
///
/// A tuple of `(Option<MathAst>, Vec<MathDiagnostic>)`:
/// - `Some(ast)` when parsing succeeded (possibly with accumulated warnings).
/// - `None` when the source is entirely unparseable.
/// - The diagnostic list is non-empty when any unsupported command or parse
///   error was encountered.
///
/// # Errors
///
/// Returns diagnostics for unsupported commands, unmatched braces, and other
/// structural errors. Never panics — all recoverable paths produce diagnostics.
#[must_use]
pub fn parse(
    latex: &str,
    mode: MathMode,
    span: SourceSpan,
) -> (Option<MathAst>, Vec<MathDiagnostic>) {
    let mut parser = LatexParser::new(latex, span);
    let nodes = parser.parse_node_list();
    let diags = parser.into_diagnostics();
    (Some(MathAst::new(mode, nodes)), diags)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal recursive-descent parser
// ─────────────────────────────────────────────────────────────────────────────

struct LatexParser<'a> {
    input: &'a str,
    pos: usize,
    span: SourceSpan,
    diags: Vec<MathDiagnostic>,
    /// Current brace-nesting depth. Incremented on every `{` and decremented
    /// on every `}`. When depth would exceed [`MAX_DEPTH`], parsing of the
    /// group is aborted and a diagnostic is emitted.
    depth: usize,
}

impl<'a> LatexParser<'a> {
    fn new(input: &'a str, span: SourceSpan) -> Self {
        LatexParser {
            input,
            pos: 0,
            span,
            diags: Vec::new(),
            depth: 0,
        }
    }

    fn into_diagnostics(self) -> Vec<MathDiagnostic> {
        self.diags
    }

    /// Remaining unparsed input.
    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    /// Skip ASCII whitespace.
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len()
            && self.input.as_bytes().get(self.pos).is_some_and(u8::is_ascii_whitespace)
        {
            self.pos += 1;
        }
    }

    /// Peek at the next byte (ASCII-safe).
    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    /// Consume one Unicode scalar value (char) from the input.
    ///
    /// Advances `self.pos` by the correct UTF-8 byte length of the consumed
    /// character, avoiding corruption of multi-byte sequences.
    fn consume_char(&mut self) -> Option<char> {
        let ch = self.input[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    /// Parse a command name after a backslash (e.g. `\alpha` → `"alpha"`).
    fn parse_command_name(&mut self) -> &'a str {
        let start = self.pos;
        // Single non-alpha character command (e.g. `\,`)
        if let Some(&b) = self.input.as_bytes().get(self.pos)
            && !b.is_ascii_alphabetic()
        {
            self.pos += 1;
            return &self.input[start..self.pos];
        }
        while let Some(&b) = self.input.as_bytes().get(self.pos) {
            if b.is_ascii_alphabetic() {
                self.pos += 1;
            } else {
                break;
            }
        }
        &self.input[start..self.pos]
    }

    /// Parse a braced group `{...}` and return the inner nodes.
    ///
    /// If the current depth would exceed [`MAX_DEPTH`], emit a diagnostic and
    /// skip to the matching `}` without recursing, preventing stack exhaustion.
    fn parse_group(&mut self) -> Vec<MathNode> {
        // Consume the opening `{`
        self.pos += 1;
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            // Emit a depth-exceeded diagnostic.
            self.diags.push(MathDiagnostic::new(
                MathRendererError::ParseError {
                    message: Arc::from(
                        "brace nesting depth exceeded maximum allowed depth (256); \
                         group content skipped",
                    ),
                    span: self.span.clone(),
                },
                self.span.clone(),
            ));
            // Skip to the matching `}` at this level without further recursion.
            // We track our own brace count to find the correct closing brace.
            let mut open_count = 1usize;
            while self.pos < self.input.len() {
                match self.input.as_bytes()[self.pos] {
                    b'{' => {
                        open_count += 1;
                        self.pos += 1;
                    }
                    b'}' => {
                        open_count -= 1;
                        if open_count == 0 {
                            self.pos += 1; // consume the closing `}`
                            break;
                        }
                        self.pos += 1;
                    }
                    _ => {
                        self.pos += 1;
                    }
                }
            }
            self.depth -= 1;
            return Vec::new();
        }

        let inner = self.parse_until(|p| p.peek_byte() == Some(b'}'));
        // Consume the closing `}` if present
        if self.peek_byte() == Some(b'}') {
            self.pos += 1;
        }
        self.depth -= 1;
        inner
    }

    /// Parse an optional bracketed index `[...]` and return inner nodes.
    fn parse_optional_bracket(&mut self) -> Option<Vec<MathNode>> {
        self.skip_whitespace();
        if self.peek_byte() == Some(b'[') {
            self.pos += 1; // consume `[`
            let inner = self.parse_until(|p| p.peek_byte() == Some(b']'));
            if self.peek_byte() == Some(b']') {
                self.pos += 1;
            }
            Some(inner)
        } else {
            None
        }
    }

    /// Parse nodes until `stop_cond` returns true or end of input.
    fn parse_until<F: Fn(&Self) -> bool>(&mut self, stop_cond: F) -> Vec<MathNode> {
        let mut nodes = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() || stop_cond(self) {
                break;
            }
            if let Some(node) = self.parse_one() {
                nodes = self.apply_scripts(nodes, node);
            }
        }
        nodes
    }

    /// Parse a list of nodes to end of input.
    fn parse_node_list(&mut self) -> Vec<MathNode> {
        self.parse_until(|_| false)
    }

    /// After parsing `node`, check if the next char is `^` or `_` and wrap
    /// appropriately.  Handles `base^{sup}`, `base_{sub}`, and combined
    /// `base^{sup}_{sub}` / `base_{sub}^{sup}`.
    ///
    /// For `Operator` nodes (large operators like `\sum`, `\prod`, `\int`), the
    /// operator is pushed to the output list first (so it appears directly in
    /// `ast.nodes`), and any following `_` / `^` scripts are produced as
    /// `Subscript`/`Superscript` nodes with an empty group base. This matches
    /// the BC-5.29.001 parser contract which requires the operator to be directly
    /// visible in the top-level node list.
    fn apply_scripts(&mut self, mut prev: Vec<MathNode>, node: MathNode) -> Vec<MathNode> {
        // Large operators are pushed immediately so they appear at the top level.
        let (is_operator, mut base) = match &node {
            MathNode::Operator(_) => {
                // Check whether scripts follow before deciding
                self.skip_whitespace();
                let has_script = matches!(self.peek_byte(), Some(b'^' | b'_'));
                if has_script {
                    // Push the operator first, then attach scripts to an empty base
                    prev.push(node);
                    (true, MathNode::Group(Vec::new()))
                } else {
                    (false, node)
                }
            }
            _ => (false, node),
        };
        let _ = is_operator; // used implicitly by the logic above

        loop {
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b'^') => {
                    self.pos += 1;
                    let sup = self.parse_single_atom();
                    base = MathNode::Superscript {
                        base: Box::new(base),
                        sup: Box::new(sup),
                    };
                }
                Some(b'_') => {
                    self.pos += 1;
                    let sub = self.parse_single_atom();
                    base = MathNode::Subscript {
                        base: Box::new(base),
                        sub: Box::new(sub),
                    };
                }
                _ => break,
            }
        }

        prev.push(base);
        prev
    }

    /// Parse exactly one "atom" — either a braced group or a single token.
    fn parse_single_atom(&mut self) -> MathNode {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'{') => {
                let inner = self.parse_group();
                if inner.len() == 1 {
                    // Unwrap single-element groups so `x^{2}` → Text("2")
                    #[allow(clippy::unwrap_used)] // safe: len==1
                    inner.into_iter().next().unwrap()
                } else {
                    MathNode::Group(inner)
                }
            }
            Some(b'\\') => {
                self.pos += 1; // consume `\`
                let cmd = self.parse_command_name();
                self.dispatch_command(cmd)
            }
            _ => {
                if let Some(ch) = self.consume_char() {
                    MathNode::Text(Arc::from(ch.to_string().as_str()))
                } else {
                    MathNode::Text(Arc::from(""))
                }
            }
        }
    }

    /// Parse one top-level node (without script chaining).
    fn parse_one(&mut self) -> Option<MathNode> {
        match self.peek_byte()? {
            b'{' => {
                let inner = self.parse_group();
                Some(MathNode::Group(inner))
            }
            b'}' => {
                // Unmatched `}` — consume and skip
                self.pos += 1;
                None
            }
            b'\\' => {
                self.pos += 1; // consume `\`
                let cmd = self.parse_command_name();
                Some(self.dispatch_command(cmd))
            }
            b'^' | b'_' => {
                // Dangling script without base — produce a Text node
                let ch = self.consume_char()?;
                Some(MathNode::Text(Arc::from(ch.to_string().as_str())))
            }
            _ => {
                let ch = self.consume_char()?;
                Some(MathNode::Text(Arc::from(ch.to_string().as_str())))
            }
        }
    }

    /// Dispatch a parsed command name to the correct `MathNode` variant.
    #[allow(clippy::too_many_lines)]
    fn dispatch_command(&mut self, cmd: &'a str) -> MathNode {
        match cmd {
            // ── Structure ─────────────────────────────────────────────────
            "frac" => {
                let num_inner = self.parse_single_atom();
                let denom_inner = self.parse_single_atom();
                MathNode::Fraction {
                    num: Box::new(num_inner),
                    denom: Box::new(denom_inner),
                }
            }
            "sqrt" => {
                let index = self.parse_optional_bracket().map(|nodes| {
                    Box::new(if nodes.len() == 1 {
                        #[allow(clippy::unwrap_used)] // safe: len==1
                        nodes.into_iter().next().unwrap()
                    } else {
                        MathNode::Group(nodes)
                    })
                });
                let radicand = self.parse_single_atom();
                MathNode::Sqrt {
                    index,
                    radicand: Box::new(radicand),
                }
            }

            // ── Accents ───────────────────────────────────────────────────
            "hat" => {
                let inner = self.parse_single_atom();
                MathNode::Accent { kind: AccentKind::Hat, inner: Box::new(inner) }
            }
            "bar" => {
                let inner = self.parse_single_atom();
                MathNode::Accent { kind: AccentKind::Bar, inner: Box::new(inner) }
            }
            "tilde" => {
                let inner = self.parse_single_atom();
                MathNode::Accent { kind: AccentKind::Tilde, inner: Box::new(inner) }
            }
            "vec" => {
                let inner = self.parse_single_atom();
                MathNode::Accent { kind: AccentKind::Vec, inner: Box::new(inner) }
            }
            "dot" => {
                let inner = self.parse_single_atom();
                MathNode::Accent { kind: AccentKind::Dot, inner: Box::new(inner) }
            }
            "ddot" => {
                let inner = self.parse_single_atom();
                MathNode::Accent { kind: AccentKind::Ddot, inner: Box::new(inner) }
            }

            // ── Greek (lowercase and uppercase) ───────────────────────────
            "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "varepsilon"
            | "zeta" | "eta" | "theta" | "vartheta" | "iota" | "kappa"
            | "lambda" | "mu" | "nu" | "xi" | "pi" | "varpi" | "rho"
            | "varrho" | "sigma" | "varsigma" | "tau" | "upsilon"
            | "phi" | "varphi" | "chi" | "psi" | "omega"
            | "Alpha" | "Beta" | "Gamma" | "Delta" | "Epsilon" | "Zeta"
            | "Eta" | "Theta" | "Iota" | "Kappa" | "Lambda" | "Mu"
            | "Nu" | "Xi" | "Pi" | "Rho" | "Sigma" | "Tau" | "Upsilon"
            | "Phi" | "Chi" | "Psi" | "Omega" => {
                MathNode::Greek(Arc::from(cmd))
            }

            // ── Operators ─────────────────────────────────────────────────
            "sum" | "prod" | "int" | "lim" | "max" | "min" => {
                MathNode::Operator(Arc::from(cmd))
            }

            // ── Symbols ───────────────────────────────────────────────────
            "cdot" | "times" | "div" | "infty" | "pm" | "mp" | "leq" | "geq" | "neq"
            | "approx" | "equiv" | "in" | "notin" | "subset" | "supset"
            | "cup" | "cap" | "emptyset" | "forall" | "exists" | "partial"
            | "nabla" | "to" | "rightarrow" | "leftarrow" | "Rightarrow"
            | "Leftarrow" | "ldots" | "cdots" | "vdots" | "ddots" => {
                MathNode::Symbol(Arc::from(cmd))
            }

            // ── Spaces ────────────────────────────────────────────────────
            "," | ";" | "quad" | "qquad" => MathNode::Space,

            // ── Delimiters ────────────────────────────────────────────────
            "left" => {
                // Consume the delimiter character after `\left`
                self.skip_whitespace();
                let left_delim = self.consume_delimiter_char();
                // Parse content until `\right`
                let pre_inner_pos = self.pos;
                let inner = self.parse_until(LatexParser::is_at_right);
                // Detect unmatched \left: if we reached EOF without finding
                // `\right`, emit a diagnostic.
                let found_right = self.is_at_right();
                if !found_right {
                    // Only emit the diagnostic if we consumed any input while
                    // searching (i.e., we were actually looking for a \right).
                    let _ = pre_inner_pos; // suppress unused warning
                    self.diags.push(MathDiagnostic::new(
                        MathRendererError::ParseError {
                            message: Arc::from(
                                "unmatched \\left delimiter: no corresponding \\right found",
                            ),
                            span: self.span.clone(),
                        },
                        self.span.clone(),
                    ));
                    // Return the inner content as a group rather than a Delimiter
                    return MathNode::Group(inner);
                }
                // Consume `\right` + its delimiter
                let right_delim = if self.peek_byte() == Some(b'\\') {
                    self.pos += 1;
                    let right_cmd = self.parse_command_name();
                    if right_cmd == "right" {
                        self.skip_whitespace();
                        self.consume_delimiter_char()
                    } else {
                        Arc::from(")")
                    }
                } else {
                    Arc::from(")")
                };
                MathNode::Delimiter { left: left_delim, right: right_delim, inner }
            }
            "right" => {
                // `\right` without matching `\left` — consume delimiter, emit text
                self.skip_whitespace();
                let _d = self.consume_delimiter_char();
                MathNode::Text(Arc::from(")"))
            }

            // ── Environments ─────────────────────────────────────────────
            "begin" => {
                self.skip_whitespace();
                let env_name = self.parse_env_name();
                match env_name {
                    "align" | "align*" | "aligned" => self.parse_align_env("align"),
                    "cases" => self.parse_cases_env(),
                    other => {
                        // Unknown environment — produce diagnostic and skip
                        if !is_supported_command(other) {
                            self.diags.push(MathDiagnostic::new(
                                MathRendererError::UnsupportedCommand {
                                    command: Arc::from(other),
                                    span: self.span.clone(),
                                    hint: Arc::from(hint_for_command(other)),
                                    error_code: "E-EXP-006",
                                },
                                self.span.clone(),
                            ));
                        }
                        MathNode::Group(Vec::new())
                    }
                }
            }
            "end" => {
                // Stray `\end{...}` — consume and ignore
                self.skip_whitespace();
                let _ = self.parse_env_name();
                MathNode::Group(Vec::new())
            }

            // ── Text/font commands ────────────────────────────────────────
            // `\text{...}` produces a TextRun (upright/plain style in OMML).
            // The other font commands (\mathrm, \mathbf, etc.) also produce
            // upright text — they are all semantically "text in math mode".
            "text" | "mathrm" | "mathbf" | "mathit" | "mathbb" => {
                // Parse the braced argument as a flat string of text nodes.
                // If the argument is a braced group, collect its Text children
                // into a single TextRun; otherwise wrap the single atom.
                self.skip_whitespace();
                let text_content = if self.peek_byte() == Some(b'{') {
                    // Parse the group and flatten inner Text nodes to a string.
                    let inner_nodes = self.parse_group();
                    let mut s = String::new();
                    for n in &inner_nodes {
                        match n {
                            MathNode::Text(t) | MathNode::TextRun(t) => s.push_str(t),
                            _ => {
                                // Non-text node inside \text{} — fall back to Group.
                                return MathNode::Group(inner_nodes);
                            }
                        }
                    }
                    s
                } else {
                    // Single atom (no braces)
                    let atom = self.parse_single_atom();
                    return match atom {
                        MathNode::Text(s) => MathNode::TextRun(s),
                        other => MathNode::Group(vec![other]),
                    };
                };
                MathNode::TextRun(Arc::from(text_content.as_str()))
            }

            // ── Unsupported command ───────────────────────────────────────
            other => {
                // Consume optional braced argument so parser stays in sync
                self.skip_whitespace();
                let _ = if self.peek_byte() == Some(b'{') {
                    Some(self.parse_group())
                } else {
                    None
                };
                self.diags.push(MathDiagnostic::new(
                    MathRendererError::UnsupportedCommand {
                        command: Arc::from(other),
                        span: self.span.clone(),
                        hint: Arc::from(hint_for_command(other)),
                        error_code: "E-EXP-006",
                    },
                    self.span.clone(),
                ));
                MathNode::Group(Vec::new())
            }
        }
    }

    /// Return true when the parser is positioned at `\right` (the delimiter
    /// command), NOT at `\rightarrow` or other commands that share the prefix.
    ///
    /// The check requires that the character after `\right` is NOT an ASCII
    /// letter, so `\rightarrow` (alphabetic continuation) is excluded.
    fn is_at_right(&self) -> bool {
        let r = self.rest();
        r.starts_with("\\right")
            && !r.as_bytes().get(6).is_some_and(u8::is_ascii_alphabetic)
    }

    /// Consume a single delimiter character or command (e.g. `(`, `)`, `[`, `]`,
    /// `\{`, `\}`, `\.`, `\|`, `\langle`, `\rangle`, `\lfloor`, `\rfloor`, etc.).
    ///
    /// When a `\` is followed by a letter, the full command name is consumed
    /// (e.g., `\langle` → `"\\langle"`). When followed by a non-letter punctuation
    /// character, only that one character is consumed (e.g., `\{` → `"\\{"`).
    fn consume_delimiter_char(&mut self) -> Arc<str> {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'\\') => {
                // Mark the start of the `\` so we can return `\cmd` as a unit.
                let backslash_pos = self.pos;
                self.pos += 1; // consume `\`
                match self.input.as_bytes().get(self.pos) {
                    Some(&b) if b.is_ascii_alphabetic() => {
                        // Multi-char command: consume all letters (e.g. `langle`).
                        while let Some(&nb) = self.input.as_bytes().get(self.pos) {
                            if nb.is_ascii_alphabetic() {
                                self.pos += 1;
                            } else {
                                break;
                            }
                        }
                        Arc::from(&self.input[backslash_pos..self.pos])
                    }
                    Some(_) => {
                        // Single non-alpha char (e.g. `{`, `}`, `.`, `|`).
                        self.pos += 1;
                        Arc::from(&self.input[backslash_pos..self.pos])
                    }
                    None => Arc::from("\\"),
                }
            }
            Some(_) => {
                let ch = self.consume_char().unwrap_or('(');
                Arc::from(ch.to_string().as_str())
            }
            None => Arc::from("("),
        }
    }

    /// Parse an environment name from `{...}`.
    fn parse_env_name(&mut self) -> &'a str {
        if self.peek_byte() == Some(b'{') {
            self.pos += 1; // consume `{`
            let start = self.pos;
            while let Some(&b) = self.input.as_bytes().get(self.pos) {
                if b == b'}' {
                    break;
                }
                self.pos += 1;
            }
            let name = &self.input[start..self.pos];
            if self.peek_byte() == Some(b'}') {
                self.pos += 1;
            }
            name
        } else {
            ""
        }
    }

    /// Parse `\begin{align}...\end{align}` into `MathNode::Align`.
    fn parse_align_env(&mut self, env: &str) -> MathNode {
        let mut rows: Vec<Vec<MathNode>> = Vec::new();
        let end_marker = format!("\\end{{{env}}}");
        let end_marker_star = format!("\\end{{{env}*}}");
        let end_marker_ed = "\\end{aligned}".to_owned();

        loop {
            // Parse one row (until `\\` or `\end{...}`)
            let row = self.parse_until(|p| {
                p.rest().starts_with("\\\\")
                    || p.rest().starts_with(&end_marker)
                    || p.rest().starts_with(&end_marker_star)
                    || p.rest().starts_with(&end_marker_ed)
            });
            rows.push(row);

            if self.rest().starts_with("\\\\") {
                self.pos += 2; // consume `\\`
            } else {
                // Consume `\end{...}`
                for marker in [&end_marker, &end_marker_star, &end_marker_ed] {
                    if self.rest().starts_with(marker.as_str()) {
                        self.pos += marker.len();
                        break;
                    }
                }
                break;
            }
        }
        MathNode::Align(rows)
    }

    /// Parse `\begin{cases}...\end{cases}` into `MathNode::Cases`.
    fn parse_cases_env(&mut self) -> MathNode {
        let end_marker = "\\end{cases}";
        let mut cases: Vec<(Vec<MathNode>, Vec<MathNode>)> = Vec::new();

        loop {
            // Parse left side up to `&` or `\\` or `\end`
            let lhs = self.parse_until(|p| {
                p.peek_byte() == Some(b'&')
                    || p.rest().starts_with("\\\\")
                    || p.rest().starts_with(end_marker)
            });

            let rhs = if self.peek_byte() == Some(b'&') {
                self.pos += 1;
                self.parse_until(|p| {
                    p.rest().starts_with("\\\\") || p.rest().starts_with(end_marker)
                })
            } else {
                Vec::new()
            };

            cases.push((lhs, rhs));

            if self.rest().starts_with("\\\\") {
                self.pos += 2;
            } else {
                if self.rest().starts_with(end_marker) {
                    self.pos += end_marker.len();
                }
                break;
            }
        }
        MathNode::Cases(cases)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Commands supported in v1.0 (used by the parser and the unsupported-command
// check).
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` if `cmd` (without leading backslash) is in the supported
/// command set for slideforge v1.0.
#[must_use]
pub fn is_supported_command(cmd: &str) -> bool {
    matches!(
        cmd,
        // Structure
        "frac" | "sqrt" |
        // Accents
        "hat" | "bar" | "tilde" | "vec" | "dot" | "ddot" |
        // Greek lowercase
        "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "varepsilon" |
        "zeta" | "eta" | "theta" | "vartheta" | "iota" | "kappa" |
        "lambda" | "mu" | "nu" | "xi" | "pi" | "varpi" | "rho" |
        "varrho" | "sigma" | "varsigma" | "tau" | "upsilon" |
        "phi" | "varphi" | "chi" | "psi" | "omega" |
        // Greek uppercase
        "Alpha" | "Beta" | "Gamma" | "Delta" | "Epsilon" | "Zeta" |
        "Eta" | "Theta" | "Iota" | "Kappa" | "Lambda" | "Mu" |
        "Nu" | "Xi" | "Pi" | "Rho" | "Sigma" | "Tau" | "Upsilon" |
        "Phi" | "Chi" | "Psi" | "Omega" |
        // Operators
        "sum" | "prod" | "int" | "lim" | "max" | "min" |
        // Symbols
        "cdot" | "times" | "div" | "infty" | "pm" | "mp" | "leq" | "geq" | "neq" |
        "approx" | "equiv" | "in" | "notin" | "subset" | "supset" |
        "cup" | "cap" | "emptyset" | "forall" | "exists" | "partial" |
        "nabla" | "to" | "rightarrow" | "leftarrow" | "Rightarrow" |
        "Leftarrow" | "ldots" | "cdots" | "vdots" | "ddots" |
        // Spaces
        "," | ";" | "quad" | "qquad" |
        // Delimiters
        "left" | "right" |
        // Environments
        "begin" | "end" |
        // Text in math
        "text" | "mathrm" | "mathbf" | "mathit" | "mathbb"
    )
}

/// Returns a correction hint for a specific known-bad command, or a generic
/// hint for unknown commands.
#[must_use]
pub fn hint_for_command(cmd: &str) -> &'static str {
    match cmd {
        "newcommand" | "renewcommand" | "def" | "let" =>
            "user-defined macros require v2+",
        "usepackage" =>
            "\\usepackage is a document-level command; not valid in slideforge math",
        "DeclareMathOperator" =>
            "custom operator declarations require v2+",
        "include" | "input" =>
            "file inclusion is not supported inside math expressions",
        "tikzpicture" =>
            "TikZ drawings are not supported in v1.0",
        _ =>
            "this command is not in the slideforge v1.0 supported LaTeX subset",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{MathMode, MathNode};
    use crate::error::MathRendererError;

    // ─────────────────────────────────────────────────────────────────────────
    // BC-5.29.001 — LaTeX parser produces correct MathAst nodes
    // ─────────────────────────────────────────────────────────────────────────

    /// A simple inline expression parses to Inline mode with a Superscript node.
    #[test]
    fn test_bc_5_29_001_parse_inline_simple() {
        let (ast, diags) = parse("E = mc^2", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        assert_eq!(ast.mode, MathMode::Inline);
        // Must contain a Superscript node
        let has_sup = ast.nodes.iter().any(|n| matches!(n, MathNode::Superscript { .. }));
        assert!(has_sup, "expected Superscript node in: {ast:?}");
    }

    /// A display-mode sum with sub+superscript parses correctly.
    #[test]
    fn test_bc_5_29_001_parse_display_sum() {
        let (ast, diags) = parse(r"\sum_{i=0}^{n} i", MathMode::Display, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        assert_eq!(ast.mode, MathMode::Display);
        // Must contain an Operator node for \sum
        let has_sum = ast.nodes.iter().any(|n| matches!(n, MathNode::Operator(s) if s.as_ref() == "sum"));
        assert!(has_sum, "expected Operator(sum) node in: {ast:?}");
        // Must have at least one Subscript and one Superscript
        fn has_variant(nodes: &[MathNode], pred: fn(&MathNode) -> bool) -> bool {
            nodes.iter().any(|n| {
                if pred(n) {
                    return true;
                }
                // recurse into groups
                match n {
                    MathNode::Group(inner) => has_variant(inner, pred),
                    MathNode::Subscript { base, sub } => {
                        has_variant(std::slice::from_ref(base), pred)
                            || has_variant(std::slice::from_ref(sub), pred)
                    }
                    MathNode::Superscript { base, sup } => {
                        has_variant(std::slice::from_ref(base), pred)
                            || has_variant(std::slice::from_ref(sup), pred)
                    }
                    _ => false,
                }
            })
        }
        assert!(
            has_variant(&ast.nodes, |n| matches!(n, MathNode::Subscript { .. })),
            "expected Subscript node"
        );
        assert!(
            has_variant(&ast.nodes, |n| matches!(n, MathNode::Superscript { .. })),
            "expected Superscript node"
        );
    }

    /// `\frac{a}{b}` parses to a Fraction node.
    #[test]
    fn test_bc_5_29_001_parse_fraction() {
        let (ast, diags) = parse(r"\frac{a}{b}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let has_frac = ast.nodes.iter().any(|n| matches!(n, MathNode::Fraction { .. }));
        assert!(has_frac, "expected Fraction node in: {ast:?}");
    }

    /// `\sqrt{x}` parses to a Sqrt node with no index.
    #[test]
    fn test_bc_5_29_001_parse_sqrt() {
        let (ast, diags) = parse(r"\sqrt{x}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let has_sqrt = ast.nodes.iter().any(|n| matches!(n, MathNode::Sqrt { index: None, .. }));
        assert!(has_sqrt, "expected Sqrt{{index:None}} node in: {ast:?}");
    }

    /// `\alpha + \beta` parses to two Greek nodes.
    #[test]
    fn test_bc_5_29_001_parse_greek() {
        let (ast, diags) = parse(r"\alpha + \beta", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let greek_nodes: Vec<_> = ast
            .nodes
            .iter()
            .filter(|n| matches!(n, MathNode::Greek(_)))
            .collect();
        assert_eq!(greek_nodes.len(), 2, "expected exactly 2 Greek nodes, got: {greek_nodes:?}");
    }

    /// An unknown LaTeX command produces a `MathRendererError::UnsupportedCommand` diagnostic.
    #[test]
    fn test_bc_5_29_001_unsupported_command_error() {
        let (_, diags) = parse(r"\undefinedcmd{x}", MathMode::Inline, SourceSpan::default());
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        let has_unsupported = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. } if command.as_ref() == "undefinedcmd")
        });
        assert!(has_unsupported, "expected UnsupportedCommand for \\undefinedcmd, got: {diags:?}");
    }

    /// Two unsupported commands in one expression produce two separate diagnostics
    /// (error accumulation, not fail-fast).
    #[test]
    fn test_bc_5_29_001_multiple_unsupported_accumulated() {
        let (_, diags) = parse(
            r"\badcmd{x} + \anotherbad{y}",
            MathMode::Inline,
            SourceSpan::default(),
        );
        let unsupported_count = diags
            .iter()
            .filter(|d| matches!(&d.error, MathRendererError::UnsupportedCommand { .. }))
            .count();
        assert!(
            unsupported_count >= 2,
            "expected ≥2 UnsupportedCommand diagnostics, got {unsupported_count}: {diags:?}"
        );
    }

    /// `\newcommand` produces a specific hint about v2+ support.
    #[test]
    fn test_bc_5_29_001_newcommand_specific_hint() {
        let (_, diags) = parse(r"\newcommand{\foo}{bar}", MathMode::Inline, SourceSpan::default());
        let hint_diag = diags.iter().find(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. } if command.as_ref() == "newcommand")
        });
        assert!(hint_diag.is_some(), "expected UnsupportedCommand for \\newcommand, got: {diags:?}");
        let hint_diag = hint_diag.unwrap();
        if let MathRendererError::UnsupportedCommand { hint, .. } = &hint_diag.error {
            assert!(
                hint.contains("v2+"),
                "expected hint to mention v2+, got: {hint}"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // is_supported_command + hint_for_command sanity checks
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_5_29_001_is_supported_command_known() {
        assert!(is_supported_command("frac"));
        assert!(is_supported_command("sum"));
        assert!(is_supported_command("alpha"));
    }

    #[test]
    fn test_bc_5_29_001_is_supported_command_unknown() {
        assert!(!is_supported_command("undefinedcmd"));
        assert!(!is_supported_command("newcommand"));
    }

    #[test]
    fn test_bc_5_29_001_hint_for_newcommand() {
        assert!(hint_for_command("newcommand").contains("v2+"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-009 — tikzpicture-specific hint
    // ─────────────────────────────────────────────────────────────────────────

    /// `hint_for_command("tikzpicture")` returns a TikZ-specific message.
    #[test]
    fn test_finding_009_tikzpicture_hint() {
        let hint = hint_for_command("tikzpicture");
        assert!(
            hint.contains("TikZ"),
            "tikzpicture hint must mention TikZ; got: {hint}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-007 — UnsupportedCommand must carry error_code "E-EXP-006"
    // ─────────────────────────────────────────────────────────────────────────

    /// An unsupported command produces a diagnostic with error_code "E-EXP-006".
    #[test]
    fn test_finding_007_unsupported_command_has_error_code() {
        let (_, diags) = parse(r"\badcmd", MathMode::Inline, SourceSpan::default());
        let diag = diags.iter().find(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. }
                if command.as_ref() == "badcmd")
        });
        assert!(diag.is_some(), "expected UnsupportedCommand diagnostic");
        if let Some(d) = diag {
            if let MathRendererError::UnsupportedCommand { error_code, .. } = &d.error {
                assert_eq!(*error_code, "E-EXP-006", "error_code must be E-EXP-006");
            }
        }
    }

    /// The Display impl includes the error code in the rendered string.
    #[test]
    fn test_finding_007_error_code_in_display() {
        let (_, diags) = parse(r"\unknowncmd", MathMode::Inline, SourceSpan::default());
        let diag = diags.iter().find(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { .. })
        });
        assert!(diag.is_some(), "expected UnsupportedCommand diagnostic");
        let msg = diag.unwrap().error.to_string();
        assert!(
            msg.contains("E-EXP-006"),
            "Display output must include E-EXP-006; got: {msg}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-004 — \div and \mp must parse without diagnostics
    // ─────────────────────────────────────────────────────────────────────────

    /// `\div` parses to a Symbol node with no diagnostics.
    #[test]
    fn test_div_symbol_parses() {
        let (ast, diags) = parse(r"a \div b", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics for \\div: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let has_div = ast.nodes.iter().any(|n| matches!(n, MathNode::Symbol(s) if s.as_ref() == "div"));
        assert!(has_div, "expected Symbol(div) node; got: {ast:?}");
    }

    /// `\mp` parses to a Symbol node with no diagnostics.
    #[test]
    fn test_mp_symbol_parses() {
        let (ast, diags) = parse(r"x \mp y", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics for \\mp: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let has_mp = ast.nodes.iter().any(|n| matches!(n, MathNode::Symbol(s) if s.as_ref() == "mp"));
        assert!(has_mp, "expected Symbol(mp) node; got: {ast:?}");
    }

    /// `is_supported_command` recognizes `div` and `mp`.
    #[test]
    fn test_div_mp_are_supported_commands() {
        assert!(is_supported_command("div"), "div must be in supported set");
        assert!(is_supported_command("mp"), "mp must be in supported set");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-003 — consume_char must handle multi-byte UTF-8 correctly
    // ─────────────────────────────────────────────────────────────────────────

    /// Parsing an expression that contains multi-byte Unicode characters must
    /// not corrupt the character or produce extra garbage Text nodes.
    #[test]
    fn test_consume_char_handles_multibyte_unicode() {
        // "∑" (U+2211 N-ARY SUMMATION) is a 3-byte UTF-8 sequence.
        // When used as a plain character in math (not as \sum), the parser
        // must consume it as a single Text node without corruption.
        let (ast, diags) = parse("∑ x", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        // The first text node must be exactly "∑" — not a garbled byte fragment
        let first_text = ast.nodes.iter().find_map(|n| {
            if let MathNode::Text(s) = n { Some(s.as_ref()) } else { None }
        });
        assert_eq!(first_text, Some("∑"), "expected '∑' as first Text node; got: {ast:?}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-016 — \sqrt[n]{x} parses to Sqrt with index=Some
    // ─────────────────────────────────────────────────────────────────────────

    /// `\sqrt[3]{x}` parses to a Sqrt node with `index = Some(Text("3"))`.
    #[test]
    fn test_finding_016_sqrt_with_index() {
        let (ast, diags) = parse(r"\sqrt[3]{x}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let sqrt_node = ast.nodes.iter().find_map(|n| {
            if let MathNode::Sqrt { index, radicand } = n {
                Some((index.clone(), radicand.clone()))
            } else {
                None
            }
        });
        assert!(sqrt_node.is_some(), "expected Sqrt node; got: {ast:?}");
        let (index, _radicand) = sqrt_node.unwrap();
        assert!(
            index.is_some(),
            "expected Sqrt with index=Some for \\sqrt[3]{{x}}; got index=None"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-015 — unmatched \left produces a diagnostic, not silent output
    // ─────────────────────────────────────────────────────────────────────────

    /// `\left( x + y` (missing `\right`) must produce a "unmatched \\left" diagnostic.
    #[test]
    fn test_finding_015_unmatched_left_emits_diagnostic() {
        let (_, diags) = parse(
            r"\left( x + y",
            MathMode::Inline,
            SourceSpan::default(),
        );
        let has_unmatched = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::ParseError { message, .. }
                if message.contains("unmatched") && message.contains("left"))
        });
        assert!(
            has_unmatched,
            "expected an 'unmatched \\\\left' ParseError diagnostic; got: {diags:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-013 — recursion depth limit: deeply nested braces must produce
    //               a diagnostic rather than a stack overflow
    // ─────────────────────────────────────────────────────────────────────────

    /// Deeply nested `{{{{...}}}}` beyond MAX_DEPTH must produce a diagnostic,
    /// not a stack overflow or panic.
    #[test]
    fn test_finding_013_deep_nesting_produces_diagnostic_not_panic() {
        // Build 300 levels of nesting — well beyond MAX_DEPTH = 256
        let deep: String = "{".repeat(300) + "x" + &"}".repeat(300);
        let (ast, diags) = parse(&deep, MathMode::Inline, SourceSpan::default());
        // Must not panic (return normally)
        // Must produce at least one diagnostic OR succeed with partial AST
        // (the key invariant is: no stack overflow)
        let _ = (ast, diags); // just reaching here is sufficient
    }

    /// A 300-level nested expression produces a recursion-depth diagnostic.
    #[test]
    fn test_finding_013_deep_nesting_emits_depth_exceeded_diagnostic() {
        let deep: String = "{".repeat(300) + "x" + &"}".repeat(300);
        let (_, diags) = parse(&deep, MathMode::Inline, SourceSpan::default());
        let has_depth_error = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::ParseError { message, .. }
                if message.contains("depth"))
        });
        assert!(
            has_depth_error,
            "expected a recursion-depth ParseError diagnostic for 300-deep nesting; got: {diags:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-012 — align and cases environment parsing tests
    // ─────────────────────────────────────────────────────────────────────────

    /// `\begin{align} a &= b \\ c &= d \end{align}` parses to Align with 2 rows.
    #[test]
    fn test_finding_012_parse_align() {
        let (ast, diags) = parse(
            r"\begin{align} a &= b \\ c &= d \end{align}",
            MathMode::Display,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let align = ast.nodes.iter().find(|n| matches!(n, MathNode::Align(_)));
        assert!(align.is_some(), "expected Align node; got: {ast:?}");
        if let Some(MathNode::Align(rows)) = align {
            assert_eq!(rows.len(), 2, "expected 2 rows in align; got: {}", rows.len());
        }
    }

    /// `\begin{cases} x & y > 0 \\ -x & y \leq 0 \end{cases}` parses to Cases.
    #[test]
    fn test_finding_012_parse_cases() {
        let (ast, diags) = parse(
            r"\begin{cases} x & y > 0 \\ -x & y \leq 0 \end{cases}",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let cases = ast.nodes.iter().find(|n| matches!(n, MathNode::Cases(_)));
        assert!(cases.is_some(), "expected Cases node; got: {ast:?}");
        if let Some(MathNode::Cases(case_list)) = cases {
            assert_eq!(case_list.len(), 2, "expected 2 cases; got: {}", case_list.len());
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-011 — consume_delimiter_char must handle multi-char backslash
    //               delimiters like \langle, \rangle, \|, \lfloor, etc.
    // ─────────────────────────────────────────────────────────────────────────

    /// `\left\langle x \right\rangle` parses to a Delimiter with left=`\langle`
    /// and right=`\rangle` inner nodes.
    #[test]
    fn test_finding_011_langle_rangle_delimiter() {
        let (ast, diags) = parse(
            r"\left\langle x \right\rangle",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let delim = ast.nodes.iter().find_map(|n| {
            if let MathNode::Delimiter { left, right, .. } = n {
                Some((left.clone(), right.clone()))
            } else {
                None
            }
        });
        assert!(delim.is_some(), "expected Delimiter node; got: {ast:?}");
        let (left, right) = delim.unwrap();
        assert_eq!(left.as_ref(), "\\langle", "left delimiter must be \\langle; got: {left}");
        assert_eq!(right.as_ref(), "\\rangle", "right delimiter must be \\rangle; got: {right}");
    }

    /// `\left\| x \right\|` parses to a Delimiter with left=`\|` and right=`\|`.
    #[test]
    fn test_finding_011_norm_delimiter() {
        let (ast, diags) = parse(
            r"\left\| x \right\|",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let delim = ast.nodes.iter().find_map(|n| {
            if let MathNode::Delimiter { left, right, .. } = n {
                Some((left.clone(), right.clone()))
            } else {
                None
            }
        });
        assert!(delim.is_some(), "expected Delimiter node; got: {ast:?}");
        let (left, right) = delim.unwrap();
        assert_eq!(left.as_ref(), "\\|", "left delimiter must be \\|; got: {left}");
        assert_eq!(right.as_ref(), "\\|", "right delimiter must be \\|; got: {right}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-001 — is_at_right must NOT match \rightarrow
    // ─────────────────────────────────────────────────────────────────────────

    /// `\left( f: A \rightarrow B \right)` must parse with `\rightarrow` as a
    /// Symbol node inside the delimiter, not as a premature `\right` match.
    #[test]
    fn test_is_at_right_does_not_match_rightarrow() {
        let (ast, diags) =
            parse(r"\left( f: A \rightarrow B \right)", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        // Top-level node must be a Delimiter (not a partial/broken parse)
        let has_delim = ast.nodes.iter().any(|n| matches!(n, MathNode::Delimiter { .. }));
        assert!(has_delim, "expected Delimiter node; got: {ast:?}");
        // The delimiter's inner nodes must contain a Symbol("rightarrow") node
        fn find_rightarrow(nodes: &[MathNode]) -> bool {
            for node in nodes {
                match node {
                    MathNode::Symbol(s) if s.as_ref() == "rightarrow" => return true,
                    MathNode::Delimiter { inner, .. } => {
                        if find_rightarrow(inner) {
                            return true;
                        }
                    }
                    MathNode::Group(inner) => {
                        if find_rightarrow(inner) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        assert!(
            find_rightarrow(&ast.nodes),
            "expected Symbol(rightarrow) inside delimiter; got: {ast:?}"
        );
    }
}
