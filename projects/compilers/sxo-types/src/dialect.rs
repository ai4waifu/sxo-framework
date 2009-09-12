//! Dialect tags and auto-detection.

/// Dialect selector for parse / render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    /// Simple-math / SM (not on current delivery route).
    #[default]
    SimpleMath,
    /// Mathematica / Wolfram forms.
    Mathematica,
    /// MATLAB forms.
    Matlab,
    /// Heuristic auto detection.
    Auto,
}

/// Detect dialect from input text (`Auto` heuristics).
pub fn detect_dialect(input: &str) -> Dialect {
    let s = input.trim();
    let wolfram = ["D[", "Integrate[", "Simplify[", "Expand[", "Factor[", "Sin[", "Cos[", "Tan[", "Log[", "Exp[", "Sqrt["];
    if wolfram.iter().any(|p| s.contains(p)) {
        return Dialect::Mathematica;
    }
    let matlab = ["diff(", "sin(", "cos(", "tan(", "exp(", "log("];
    if matlab.iter().any(|p| s.contains(p)) && !s.contains('[') {
        return Dialect::Matlab;
    }
    Dialect::SimpleMath
}
