use super::*;

use std::collections::BTreeMap;

use crate::postmortem::postmortem_threshold as postmortem_postmortem_threshold;

pub fn summarize_fallback_postmortem_metrics_for_group(
    items: &[postmortem_postmortem_threshold::FallbackPostmortemMetric],
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

pub fn derive_fallback_page_groups_from_metric(
    items: &[postmortem_postmortem_threshold::FallbackPostmortemMetric],
) -> Vec<FallbackPageGroup> {
    let mut out: Vec<FallbackPageGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_page_group(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_fallback_page_group(item: &FallbackPageGroup) -> String {
    postmortem_postmortem_threshold::classify_scheduled_postmortem_schedule(item.weighted_value())
}
