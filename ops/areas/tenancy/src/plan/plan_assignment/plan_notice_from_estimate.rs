use super::*;

use std::collections::BTreeMap;

use crate::isolation::estimate as isolation_estimate;

pub fn summarize_seasonal_isolation_estimates_for_notice(
    items: &[isolation_estimate::SeasonalIsolationEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_plan_notices_from_estimate(items: &[isolation_estimate::SeasonalIsolationEstimate]) -> Vec<PlanNotice> {
    let mut out: Vec<PlanNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_notice(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_plan_notice(item: &PlanNotice) -> f64 {
    isolation_estimate::seasonal_isolation_estimate_weight(9, true)
}
