use super::*;

use std::collections::BTreeMap;

use crate::evidence::evidence_claim as evidence_evidence_claim;

pub fn summarize_capped_evidence_allocations_for_profile(
    items: &[evidence_evidence_claim::CappedEvidenceAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_primary_quote_profiles_from_allocation(
    items: &[evidence_evidence_claim::CappedEvidenceAllocation],
) -> Vec<PrimaryQuoteProfile> {
    let mut out: Vec<PrimaryQuoteProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_quote_profile(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_primary_quote_profile(item: &PrimaryQuoteProfile) -> String {
    evidence_evidence_claim::normalize_lenient_evidence_plan_code(&item.code)
}
