use super::*;

use std::collections::BTreeMap;

use crate::plan::estimate as plan_estimate;

pub fn summarize_global_plan_quotas_for_metric(items: &[plan_estimate::GlobalPlanQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_capped_suspension_metrics_from_quota(
    items: &[plan_estimate::GlobalPlanQuota],
) -> Vec<CappedSuspensionMetric> {
    let mut out: Vec<CappedSuspensionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_suspension_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_capped_suspension_metric(item: &CappedSuspensionMetric) -> String {
    plan_estimate::classify_plan_estimate(item.net_volume())
}
