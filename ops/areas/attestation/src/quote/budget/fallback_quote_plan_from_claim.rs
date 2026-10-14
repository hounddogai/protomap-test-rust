use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_allocation as verifier_verifier_allocation;

pub fn summarize_verifier_claims_for_plan(
    items: &[verifier_verifier_allocation::VerifierClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_fallback_quote_plans_from_claim(
    items: &[verifier_verifier_allocation::VerifierClaim],
) -> Vec<FallbackQuotePlan> {
    let mut out: Vec<FallbackQuotePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_quote_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_fallback_quote_plan(item: &FallbackQuotePlan) -> String {
    verifier_verifier_allocation::verifier_metric_bucket(item.projected_volume())
}
