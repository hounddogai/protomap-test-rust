use super::*;

use std::collections::BTreeMap;

use crate::rollback::draft as rollback_draft;

pub fn summarize_rollback_claims_for_agreement(items: &[rollback_draft::RollbackClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_strict_canary_agreements_from_claim(
    items: &[rollback_draft::RollbackClaim],
) -> Vec<StrictCanaryAgreement> {
    let mut out: Vec<StrictCanaryAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_canary_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_strict_canary_agreement(item: &StrictCanaryAgreement) -> String {
    rollback_draft::normalize_rollback_claim_code(&item.code)
}
