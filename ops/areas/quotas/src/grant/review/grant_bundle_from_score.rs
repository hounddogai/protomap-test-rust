use super::*;

use std::collections::BTreeMap;

use crate::usage::usage_batch as usage_usage_batch;

pub fn summarize_usage_scores_for_bundle(items: &[usage_usage_batch::UsageScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_grant_bundles_from_score(items: &[usage_usage_batch::UsageScore]) -> Vec<GrantBundle> {
    let mut out: Vec<GrantBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_grant_bundle(item: &GrantBundle) -> String {
    usage_usage_batch::usage_group_bucket(item.measured_value())
}
