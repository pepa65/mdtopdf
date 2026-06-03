//! LaTeX to Typst math conversion
//!
//! Converts LaTeX math expressions to Typst's native math syntax.
//! Typst uses a cleaner, more readable math syntax than LaTeX.

use regex::Regex;
use std::borrow::Cow;

/// Convert a LaTeX math expression to Typst math syntax
pub fn latex_to_typst_math(latex: &str) -> String {
    let mut result = latex.to_string();

    // Apply all conversions
    result = convert_fractions(&result);
    result = convert_roots(&result);
    result = convert_superscripts_subscripts(&result);
    result = convert_greek_letters(&result);
    result = convert_operators(&result);
    result = convert_delimiters(&result);
    result = convert_functions(&result);
    result = convert_accents(&result);
    result = convert_misc(&result);

    // Clean up extra braces and whitespace
    result = cleanup(&result);

    result
}

/// Convert \frac{a}{b} to frac(a, b)
fn convert_fractions(input: &str) -> String {
    let re = Regex::new(r"\\frac\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}")
        .unwrap();
    re.replace_all(input, "frac($1, $2)").to_string()
}

/// Convert \sqrt{x} to sqrt(x) and \sqrt[n]{x} to root(n, x)
fn convert_roots(input: &str) -> String {
    // \sqrt[n]{x} -> root(n, x)
    let re_nth = Regex::new(r"\\sqrt\[([^\]]+)\]\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap();
    let result = re_nth.replace_all(input, "root($1, $2)").to_string();

    // \sqrt{x} -> sqrt(x)
    let re_sqrt = Regex::new(r"\\sqrt\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}").unwrap();
    re_sqrt.replace_all(&result, "sqrt($1)").to_string()
}

/// Convert superscripts and subscripts
fn convert_superscripts_subscripts(input: &str) -> String {
    let mut result = input.to_string();

    // x^{exp} -> x^(exp) for multi-char exponents
    let re_sup = Regex::new(r"\^\{([^{}]+)\}").unwrap();
    result = re_sup.replace_all(&result, "^($1)").to_string();

    // x_{sub} -> x_(sub) for multi-char subscripts
    let re_sub = Regex::new(r"_\{([^{}]+)\}").unwrap();
    result = re_sub.replace_all(&result, "_($1)").to_string();

    result
}

/// Convert Greek letters
fn convert_greek_letters(input: &str) -> String {
    let greek_map = [
        // Lowercase
        (r"\alpha", "alpha"),
        (r"\beta", "beta"),
        (r"\gamma", "gamma"),
        (r"\delta", "delta"),
        (r"\epsilon", "epsilon"),
        (r"\varepsilon", "epsilon.alt"),
        (r"\zeta", "zeta"),
        (r"\eta", "eta"),
        (r"\theta", "theta"),
        (r"\vartheta", "theta.alt"),
        (r"\iota", "iota"),
        (r"\kappa", "kappa"),
        (r"\lambda", "lambda"),
        (r"\mu", "mu"),
        (r"\nu", "nu"),
        (r"\xi", "xi"),
        (r"\pi", "pi"),
        (r"\varpi", "pi.alt"),
        (r"\rho", "rho"),
        (r"\varrho", "rho.alt"),
        (r"\sigma", "sigma"),
        (r"\varsigma", "sigma.alt"),
        (r"\tau", "tau"),
        (r"\upsilon", "upsilon"),
        (r"\phi", "phi"),
        (r"\varphi", "phi.alt"),
        (r"\chi", "chi"),
        (r"\psi", "psi"),
        (r"\omega", "omega"),
        // Uppercase
        (r"\Gamma", "Gamma"),
        (r"\Delta", "Delta"),
        (r"\Theta", "Theta"),
        (r"\Lambda", "Lambda"),
        (r"\Xi", "Xi"),
        (r"\Pi", "Pi"),
        (r"\Sigma", "Sigma"),
        (r"\Upsilon", "Upsilon"),
        (r"\Phi", "Phi"),
        (r"\Psi", "Psi"),
        (r"\Omega", "Omega"),
    ];

    let mut result = input.to_string();
    for (latex, typst) in greek_map {
        result = result.replace(latex, typst);
    }
    result
}

/// Convert common operators
fn convert_operators(input: &str) -> String {
    let op_map = [
        // Big operators
        (r"\sum", "sum"),
        (r"\prod", "product"),
        (r"\coprod", "coproduct"),
        (r"\int", "integral"),
        (r"\iint", "integral.double"),
        (r"\iiint", "integral.triple"),
        (r"\oint", "integral.cont"),
        (r"\bigcup", "union.big"),
        (r"\bigcap", "sect.big"),
        (r"\bigoplus", "plus.circle.big"),
        (r"\bigotimes", "times.circle.big"),
        // Binary operators
        (r"\times", "times"),
        (r"\div", "div"),
        (r"\cdot", "dot"),
        (r"\pm", "plus.minus"),
        (r"\mp", "minus.plus"),
        (r"\oplus", "plus.circle"),
        (r"\otimes", "times.circle"),
        (r"\cup", "union"),
        (r"\cap", "sect"),
        (r"\setminus", "without"),
        (r"\circ", "compose"),
        // Relations
        (r"\leq", "<="),
        (r"\le", "<="),
        (r"\geq", ">="),
        (r"\ge", ">="),
        (r"\neq", "!="),
        (r"\ne", "!="),
        (r"\approx", "approx"),
        (r"\equiv", "equiv"),
        (r"\sim", "tilde"),
        (r"\simeq", "tilde.eq"),
        (r"\cong", "tilde.equiv"),
        (r"\propto", "prop"),
        (r"\subset", "subset"),
        (r"\supset", "supset"),
        (r"\subseteq", "subset.eq"),
        (r"\supseteq", "supset.eq"),
        (r"\in", "in"),
        (r"\notin", "in.not"),
        (r"\ni", "in.rev"),
        // Arrows
        (r"\rightarrow", "->"),
        (r"\to", "->"),
        (r"\leftarrow", "<-"),
        (r"\leftrightarrow", "<->"),
        (r"\Rightarrow", "=>"),
        (r"\Leftarrow", "<="),
        (r"\Leftrightarrow", "<=>"),
        (r"\mapsto", "|->"),
        (r"\uparrow", "arrow.t"),
        (r"\downarrow", "arrow.b"),
        // Logic
        (r"\land", "and"),
        (r"\wedge", "and"),
        (r"\lor", "or"),
        (r"\vee", "or"),
        (r"\lnot", "not"),
        (r"\neg", "not"),
        (r"\forall", "forall"),
        (r"\exists", "exists"),
        (r"\nexists", "exists.not"),
        // Misc symbols
        (r"\infty", "infinity"),
        (r"\partial", "diff"),
        (r"\nabla", "nabla"),
        (r"\emptyset", "emptyset"),
        (r"\varnothing", "nothing"),
        (r"\ldots", "..."),
        (r"\cdots", "dots.c"),
        (r"\vdots", "dots.v"),
        (r"\ddots", "dots.down"),
        (r"\prime", "'"),
        (r"\ell", "ell"),
        (r"\hbar", "planck.reduce"),
        (r"\Re", "Re"),
        (r"\Im", "Im"),
        (r"\aleph", "aleph"),
    ];

    let mut result = input.to_string();
    for (latex, typst) in op_map {
        result = result.replace(latex, typst);
    }
    result
}

/// Convert delimiters
fn convert_delimiters(input: &str) -> String {
    let delim_map = [
        (r"\left(", "("),
        (r"\right)", ")"),
        (r"\left[", "["),
        (r"\right]", "]"),
        (r"\left\{", "{"),
        (r"\right\}", "}"),
        (r"\left|", "|"),
        (r"\right|", "|"),
        (r"\left\|", "||"),
        (r"\right\|", "||"),
        (r"\langle", "angle.l"),
        (r"\rangle", "angle.r"),
        (r"\lfloor", "floor.l"),
        (r"\rfloor", "floor.r"),
        (r"\lceil", "ceil.l"),
        (r"\rceil", "ceil.r"),
        (r"\left.", ""),
        (r"\right.", ""),
        // Simple versions without \left/\right
        (r"\{", "{"),
        (r"\}", "}"),
        (r"\|", "||"),
    ];

    let mut result = input.to_string();
    for (latex, typst) in delim_map {
        result = result.replace(latex, typst);
    }
    result
}

/// Convert common functions
fn convert_functions(input: &str) -> String {
    let func_map = [
        (r"\sin", "sin"),
        (r"\cos", "cos"),
        (r"\tan", "tan"),
        (r"\cot", "cot"),
        (r"\sec", "sec"),
        (r"\csc", "csc"),
        (r"\arcsin", "arcsin"),
        (r"\arccos", "arccos"),
        (r"\arctan", "arctan"),
        (r"\sinh", "sinh"),
        (r"\cosh", "cosh"),
        (r"\tanh", "tanh"),
        (r"\log", "log"),
        (r"\ln", "ln"),
        (r"\exp", "exp"),
        (r"\lim", "lim"),
        (r"\limsup", "limsup"),
        (r"\liminf", "liminf"),
        (r"\max", "max"),
        (r"\min", "min"),
        (r"\sup", "sup"),
        (r"\inf", "inf"),
        (r"\det", "det"),
        (r"\dim", "dim"),
        (r"\ker", "ker"),
        (r"\deg", "deg"),
        (r"\gcd", "gcd"),
        (r"\arg", "arg"),
        (r"\mod", "mod"),
        (r"\bmod", "mod"),
        (r"\pmod", "mod"),
    ];

    let mut result = input.to_string();
    for (latex, typst) in func_map {
        result = result.replace(latex, typst);
    }
    result
}

/// Convert accents and decorations
fn convert_accents(input: &str) -> String {
    let mut result = input.to_string();

    // \hat{x} -> hat(x)
    let accent_map = [
        (r"\\hat\{([^{}]+)\}", "hat($1)"),
        (r"\\bar\{([^{}]+)\}", "overline($1)"),
        (r"\\overline\{([^{}]+)\}", "overline($1)"),
        (r"\\underline\{([^{}]+)\}", "underline($1)"),
        (r"\\tilde\{([^{}]+)\}", "tilde($1)"),
        (r"\\vec\{([^{}]+)\}", "arrow($1)"),
        (r"\\dot\{([^{}]+)\}", "dot($1)"),
        (r"\\ddot\{([^{}]+)\}", "dot.double($1)"),
        (r"\\acute\{([^{}]+)\}", "acute($1)"),
        (r"\\grave\{([^{}]+)\}", "grave($1)"),
        (r"\\breve\{([^{}]+)\}", "breve($1)"),
        (r"\\check\{([^{}]+)\}", "caron($1)"),
    ];

    for (pattern, replacement) in accent_map {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }

    result
}

/// Convert miscellaneous LaTeX constructs
fn convert_misc(input: &str) -> String {
    let mut result = input.to_string();

    // \text{...} -> "..."
    if let Ok(re) = Regex::new(r#"\\text\{([^{}]+)\}"#) {
        result = re.replace_all(&result, "\"$1\"").to_string();
    }

    // \mathrm{...} -> upright($1)
    if let Ok(re) = Regex::new(r"\\mathrm\{([^{}]+)\}") {
        result = re.replace_all(&result, "upright($1)").to_string();
    }

    // \mathbf{...} -> bold($1)
    if let Ok(re) = Regex::new(r"\\mathbf\{([^{}]+)\}") {
        result = re.replace_all(&result, "bold($1)").to_string();
    }

    // \mathit{...} -> italic($1)
    if let Ok(re) = Regex::new(r"\\mathit\{([^{}]+)\}") {
        result = re.replace_all(&result, "italic($1)").to_string();
    }

    // \mathcal{...} -> cal($1)
    if let Ok(re) = Regex::new(r"\\mathcal\{([^{}]+)\}") {
        result = re.replace_all(&result, "cal($1)").to_string();
    }

    // \mathbb{...} -> bb($1)
    if let Ok(re) = Regex::new(r"\\mathbb\{([^{}]+)\}") {
        result = re.replace_all(&result, "bb($1)").to_string();
    }

    // \binom{n}{k} -> binom(n, k)
    if let Ok(re) = Regex::new(r"\\binom\{([^{}]+)\}\{([^{}]+)\}") {
        result = re.replace_all(&result, "binom($1, $2)").to_string();
    }

    // Remove \displaystyle, \textstyle, etc.
    let style_commands = [
        r"\displaystyle",
        r"\textstyle",
        r"\scriptstyle",
        r"\scriptscriptstyle",
    ];
    for cmd in style_commands {
        result = result.replace(cmd, "");
    }

    // Remove spacing commands
    let spacing = [
        r"\,", r"\;", r"\:", r"\!", r"\ ", r"\quad", r"\qquad", r"\hspace{1em}",
    ];
    for sp in spacing {
        result = result.replace(sp, " ");
    }

    // \\ for line breaks in aligned equations
    result = result.replace(r"\\", "\n");

    // & for alignment points - Typst uses different syntax
    result = result.replace(" & ", " ");
    result = result.replace("&", " ");

    result
}

/// Clean up the result
fn cleanup(input: &str) -> String {
    let mut result = input.to_string();

    // Remove empty braces
    result = result.replace("{}", "");

    // Add spaces between consecutive lowercase letters for multiplication
    // In Typst, `mc` means a variable named "mc", but in LaTeX it means m*c
    // We need to add spaces: `mc` -> `m c`
    // BUT we must preserve known function names like sqrt, frac, sin, etc.
    // Strategy: only split 2-letter sequences that aren't known identifiers
    let known_identifiers = [
        "in", "or", "if", "to", "pi", "ln", "lg", "Re", "Im",
    ];

    // Split 2-letter sequences that aren't known identifiers
    // Pattern: a single letter followed by a single letter, where neither is part of a longer word
    if let Ok(re) = Regex::new(r"\b([a-z])([a-z])\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let full = format!("{}{}", &caps[1], &caps[2]);
                if known_identifiers.contains(&full.as_str()) {
                    full
                } else {
                    format!("{} {}", &caps[1], &caps[2])
                }
            })
            .to_string();
    }

    // Normalize whitespace
    let re = Regex::new(r"\s+").unwrap();
    result = re.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

/// Process inline math ($...$) in text
pub fn process_inline_math(text: &str) -> Cow<'_, str> {
    let re = Regex::new(r"\$([^$]+)\$").unwrap();

    if !re.is_match(text) {
        return Cow::Borrowed(text);
    }

    Cow::Owned(
        re.replace_all(text, |caps: &regex::Captures| {
            let latex = &caps[1];
            let typst = latex_to_typst_math(latex);
            format!("${}$", typst)
        })
        .to_string(),
    )
}

/// Process display math ($$...$$) in text
pub fn process_display_math(text: &str) -> Cow<'_, str> {
    let re = Regex::new(r"\$\$([^$]+)\$\$").unwrap();

    if !re.is_match(text) {
        return Cow::Borrowed(text);
    }

    Cow::Owned(
        re.replace_all(text, |caps: &regex::Captures| {
            let latex = &caps[1];
            let typst = latex_to_typst_math(latex);
            format!("$ {} $", typst)
        })
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction() {
        assert_eq!(latex_to_typst_math(r"\frac{a}{b}"), "frac(a, b)");
        assert_eq!(latex_to_typst_math(r"\frac{x+1}{y-1}"), "frac(x+1, y-1)");
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(latex_to_typst_math(r"\sqrt{x}"), "sqrt(x)");
        assert_eq!(latex_to_typst_math(r"\sqrt[3]{x}"), "root(3, x)");
    }

    #[test]
    fn test_greek() {
        assert_eq!(latex_to_typst_math(r"\alpha + \beta"), "alpha + beta");
        assert_eq!(latex_to_typst_math(r"\Omega"), "Omega");
    }

    #[test]
    fn test_operators() {
        assert_eq!(latex_to_typst_math(r"\sum_{i=1}^{n}"), "sum_(i=1)^(n)");
        assert_eq!(latex_to_typst_math(r"\int_0^1"), "integral_0^1");
    }

    #[test]
    fn test_relations() {
        assert_eq!(latex_to_typst_math(r"a \leq b"), "a <= b");
        assert_eq!(latex_to_typst_math(r"x \in S"), "x in S");
    }

    #[test]
    fn test_complex_expression() {
        let latex = r"E = mc^{2}";
        let typst = latex_to_typst_math(latex);
        // In Typst, consecutive letters need spaces for multiplication
        assert_eq!(typst, "E = m c^(2)");
    }

    #[test]
    fn test_process_inline_math() {
        let text = "The equation $E = mc^{2}$ is famous";
        let result = process_inline_math(text);
        // In Typst, consecutive letters need spaces for multiplication
        assert!(result.contains("$E = m c^(2)$"));
    }
}
