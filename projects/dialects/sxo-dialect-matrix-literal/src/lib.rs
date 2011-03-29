//! Dense `MatrixValue` construction from dialect Form list literals (Living 16).
//!
//! Dialects implement [`MatrixLiteralForm`] and call [`matrix_from_form`]. Variables and
//! computed terms are not reverse-recognized from arena `Collection`s.

use athena::{domains::linear_algebra::MatrixValue, numeric::Rational};

/// Form node that may represent a numeric matrix literal (`nested rows` or flat `1×n`).
pub trait MatrixLiteralForm: Sized {
    /// Top-level or row `List` children. `None` when not a list node.
    fn list_items(&self) -> Option<&[Self]>;

    /// Exact complex scalar `(re, im)` when the node is a single complex literal.
    fn scalar_complex(&self) -> Option<(Rational, Rational)>;

    /// Exact rational scalar.
    fn scalar_rational(&self) -> Option<Rational>;

    /// Machine float scalar (promotes the whole matrix when any cell uses it).
    fn f64_lossy(&self) -> Option<f64>;

    /// When `true`, skip `1×1` complex recognition (MATLAB `i`/`j` ambiguity).
    fn reject_bare_scalar_complex(&self) -> bool {
        false
    }
}

/// Build a dense `MatrixValue` from nested list Form literals.
///
/// Nested row lists → 2-D matrix. Flat list → `1×n` row. Exact rationals stay exact.
/// Any machine float promotes the whole matrix to machine parent.
pub fn matrix_from_form<F: MatrixLiteralForm + Clone>(w: &F) -> Option<MatrixValue> {
    if w.list_items().is_none() {
        if !w.reject_bare_scalar_complex() {
            if let Some((re, im)) = w.scalar_complex() {
                if !im.is_zero() {
                    return MatrixValue::from_complex_exact_row_major(1, 1, vec![(re, im)]).ok();
                }
            }
        }
        return None;
    }
    let rows = w.list_items()?;
    if rows.is_empty() {
        return None;
    }
    let (nrows, ncols, cells) = if rows[0].list_items().is_some() {
        let mut cells = Vec::new();
        let mut cols: Option<u64> = None;
        for row in rows {
            let row_cells = row.list_items()?;
            let c = row_cells.len() as u64;
            match cols {
                Some(prev) if prev != c => return None,
                None => cols = Some(c),
                _ => {}
            }
            cells.extend_from_slice(row_cells);
        }
        (rows.len() as u64, cols.unwrap_or(0), cells)
    }
    else {
        (1u64, rows.len() as u64, rows.to_vec())
    };
    if cells.is_empty() {
        return None;
    }
    let mut complexes = Vec::with_capacity(cells.len());
    let mut any_imag = false;
    let mut all_complex = true;
    for cell in &cells {
        match cell.scalar_complex() {
            Some((re, im)) => {
                if !im.is_zero() {
                    any_imag = true;
                }
                complexes.push((re, im));
            }
            None => {
                all_complex = false;
                break;
            }
        }
    }
    if all_complex && any_imag {
        return MatrixValue::from_complex_exact_row_major(nrows, ncols, complexes).ok();
    }
    let mut rationals = Vec::with_capacity(cells.len());
    let mut all_rational = true;
    for cell in &cells {
        match cell.scalar_rational() {
            Some(q) => rationals.push(q),
            None => {
                all_rational = false;
                break;
            }
        }
    }
    if all_rational {
        return MatrixValue::from_rationals_row_major(nrows, ncols, rationals).ok();
    }
    let mut floats = Vec::with_capacity(cells.len());
    for cell in &cells {
        floats.push(cell.f64_lossy()?);
    }
    MatrixValue::from_f64_row_major(nrows, ncols, floats).ok()
}
