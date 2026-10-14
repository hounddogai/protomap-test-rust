use super::*;

use std::collections::BTreeMap;

use crate::timeline::forecast as timeline_forecast;

pub fn summarize_primary_timeline_rules_for_variance(
    items: &[timeline_forecast::PrimaryTimelineRule],
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

pub fn derive_timeline_variances_from_rule(items: &[timeline_forecast::PrimaryTimelineRule]) -> Vec<TimelineVariance> {
    let mut out: Vec<TimelineVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_variance(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_timeline_variance(item: &TimelineVariance) -> String {
    timeline_forecast::default_timeline_notice_bucket(item.projected_value())
}
