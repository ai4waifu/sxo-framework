//! Explicit dialect tags (no auto-detection).

/// Dialect selector for parse / render.
///
/// Callers must choose a concrete dialect. There is no `Auto` variant and no
/// heuristic detection of dialect from source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    /// Simple-math / SM (not on current delivery route).
    #[default]
    SimpleMath,
    /// Mathematica / Wolfram forms.
    Mathematica,
    /// MATLAB forms.
    Matlab,
}
