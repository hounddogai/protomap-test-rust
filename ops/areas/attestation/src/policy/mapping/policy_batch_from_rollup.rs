use super::*;

use std::collections::BTreeMap;

use crate::verifier::score as verifier_score;

pub fn summarize_strict_verifier_rollups_for_batch(
    items: &[verifier_score::StrictVerifierRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_policy_batches_from_rollup(items: &[verifier_score::StrictVerifierRollup]) -> Vec<PolicyBatch> {
    let mut out: Vec<PolicyBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_batch(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_policy_batch(item: &PolicyBatch) -> String {
    verifier_score::normalize_strict_verifier_rollup_code(&item.code)
}
