use super::*;

use std::collections::BTreeMap;

use crate::overage::estimate as overage_estimate;

pub fn summarize_pending_overage_batches_for_plan(
    items: &[overage_estimate::PendingOverageBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_lenient_overage_plans_from_batch(
    items: &[overage_estimate::PendingOverageBatch],
) -> Vec<LenientOveragePlan> {
    let mut out: Vec<LenientOveragePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_overage_plan(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_lenient_overage_plan(item: &LenientOveragePlan) -> f64 {
    overage_estimate::overage_estimate_weight(2, true)
}
