//! Summaries for Athena result conditions (host product surface).

use athena_types::{Condition, Predicate};

/// Stable machine-oriented summary for one [Condition] (no Term pretty-print).
pub fn condition_summary(condition: &Condition) -> String {
    let kind = predicate_kind(&condition.predicate);
    format!("{kind} resolved={}", condition.resolved)
}

fn predicate_kind(predicate: &Predicate) -> &'static str {
    match predicate {
        Predicate::Equal(_, _) => "Equal",
        Predicate::NotEqual(_, _) => "NotEqual",
        Predicate::Less(_, _) => "Less",
        Predicate::LessEqual(_, _) => "LessEqual",
        Predicate::Greater(_, _) => "Greater",
        Predicate::GreaterEqual(_, _) => "GreaterEqual",
        Predicate::Integer(_) => "Integer",
        Predicate::Positive(_) => "Positive",
        Predicate::NonNegative(_) => "NonNegative",
        Predicate::Real(_) => "Real",
        Predicate::Complex(_) => "Complex",
        Predicate::NonZero(_) => "NonZero",
        Predicate::SymbolNonZero(_) => "SymbolNonZero",
        Predicate::SymbolReal(_) => "SymbolReal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_types::{Condition, Predicate, SymbolId};

    #[test]
    fn condition_summary_names_predicate_kind() {
        let c = Condition {
            predicate: Predicate::SymbolReal(SymbolId(7)),
            resolved: false,
        };
        assert_eq!(condition_summary(&c), "SymbolReal resolved=false");
    }
}
