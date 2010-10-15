//! Summaries for Athena result evidence (host product surface).

/// Stable machine-oriented summary for one trusted-kernel evidence row.
///
/// Does not pretty-print Terms. Hosts pass provider machine name + kernel summary text.
pub fn trusted_kernel_evidence_summary(provider_name: &str, summary: &str) -> String {
    format!("TrustedKernelSummary provider={provider_name} {summary}")
}

/// Stable machine-oriented summary for one admitted-relation evidence row.
///
/// Carries only the journal `FactId` numeric. Does not reconstruct proofs or claim text.
pub fn admitted_relation_evidence_summary(fact_id: u64) -> String {
    format!("AdmittedRelation fact={fact_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_kernel_evidence_summary_is_stable() {
        assert_eq!(
            trusted_kernel_evidence_summary("Solve", "solution_rules coverage=Complete"),
            "TrustedKernelSummary provider=Solve solution_rules coverage=Complete"
        );
    }

    #[test]
    fn admitted_relation_evidence_summary_is_stable() {
        assert_eq!(admitted_relation_evidence_summary(3), "AdmittedRelation fact=3");
    }
}
