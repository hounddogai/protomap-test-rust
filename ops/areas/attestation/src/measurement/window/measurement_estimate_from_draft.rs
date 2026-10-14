use super::*;

use std::collections::BTreeMap;

use crate::policy::preference as policy_preference;

pub fn summarize_weighted_policy_drafts_for_estimate(
    items: &[policy_preference::WeightedPolicyDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_measurement_estimates_from_draft(
    items: &[policy_preference::WeightedPolicyDraft],
) -> Vec<MeasurementEstimate> {
    let mut out: Vec<MeasurementEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_estimate(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_measurement_estimate(item: &MeasurementEstimate) -> String {
    policy_preference::regional_policy_preference_bucket(item.net_load())
}
