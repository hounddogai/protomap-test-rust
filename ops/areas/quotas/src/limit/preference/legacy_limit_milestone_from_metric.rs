use super::*;

use std::collections::BTreeMap;

use crate::limit::strict_limit_metric as limit_strict_limit_metric;

pub fn summarize_fallback_limit_metrics_for_milestone(
    items: &[limit_strict_limit_metric::FallbackLimitMetric],
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

pub fn derive_legacy_limit_milestones_from_metric(
    items: &[limit_strict_limit_metric::FallbackLimitMetric],
) -> Vec<LegacyLimitMilestone> {
    let mut out: Vec<LegacyLimitMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_limit_milestone(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_legacy_limit_milestone(item: &LegacyLimitMilestone) -> f64 {
    limit_strict_limit_metric::seasonal_limit_metric_weight(1, true)
}
