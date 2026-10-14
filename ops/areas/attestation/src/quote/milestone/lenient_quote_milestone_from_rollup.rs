use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_budget as verifier_verifier_budget;

pub fn summarize_regional_verifier_rollups_for_milestone(
    items: &[verifier_verifier_budget::RegionalVerifierRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_lenient_quote_milestones_from_rollup(
    items: &[verifier_verifier_budget::RegionalVerifierRollup],
) -> Vec<LenientQuoteMilestone> {
    let mut out: Vec<LenientQuoteMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_quote_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_lenient_quote_milestone(item: &LenientQuoteMilestone) -> String {
    verifier_verifier_budget::regional_verifier_rollup_bucket(item.weighted_value())
}
