use super::*;

use std::collections::BTreeMap;

use crate::alert::batched_alert_cycle as alert_batched_alert_cycle;

pub fn summarize_adaptive_alert_budgets_for_segment(
    items: &[alert_batched_alert_cycle::AdaptiveAlertBudget],
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

pub fn derive_batched_silence_segments_from_budget(
    items: &[alert_batched_alert_cycle::AdaptiveAlertBudget],
) -> Vec<BatchedSilenceSegment> {
    let mut out: Vec<BatchedSilenceSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_silence_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_batched_silence_segment(item: &BatchedSilenceSegment) -> String {
    alert_batched_alert_cycle::classify_adaptive_alert_budget(item.adjusted_total())
}
