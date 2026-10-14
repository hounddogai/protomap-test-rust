use super::*;

use std::collections::BTreeMap;

use crate::overage::milestone as overage_milestone;

pub fn summarize_overage_digests_for_batch(items: &[overage_milestone::OverageDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_global_usage_batches_from_digest(items: &[overage_milestone::OverageDigest]) -> Vec<GlobalUsageBatch> {
    let mut out: Vec<GlobalUsageBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_global_usage_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_global_usage_batch(item: &GlobalUsageBatch) -> String {
    overage_milestone::normalize_overage_milestone_code(&item.code)
}
