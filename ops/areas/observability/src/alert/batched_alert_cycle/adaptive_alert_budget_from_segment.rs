use super::*;

use std::collections::BTreeMap;

use crate::silence::primary_silence_variance as silence_primary_silence_variance;

pub fn summarize_silence_segments_for_budget(
    items: &[silence_primary_silence_variance::SilenceSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_adaptive_alert_budgets_from_segment(
    items: &[silence_primary_silence_variance::SilenceSegment],
) -> Vec<AdaptiveAlertBudget> {
    let mut out: Vec<AdaptiveAlertBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_alert_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_adaptive_alert_budget(item: &AdaptiveAlertBudget) -> String {
    silence_primary_silence_variance::classify_silence_segment(item.net_total())
}
