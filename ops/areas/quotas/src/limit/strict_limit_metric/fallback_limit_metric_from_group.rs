use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_mapping as limit_limit_mapping;

pub fn summarize_rolling_limit_groups_for_metric(
    items: &[limit_limit_mapping::RollingLimitGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_fallback_limit_metrics_from_group(
    items: &[limit_limit_mapping::RollingLimitGroup],
) -> Vec<FallbackLimitMetric> {
    let mut out: Vec<FallbackLimitMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_limit_metric(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_fallback_limit_metric(item: &FallbackLimitMetric) -> String {
    limit_limit_mapping::rolling_limit_group_bucket(item.net_total())
}
