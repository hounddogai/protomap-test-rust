use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_adjustment as escalation_escalation_adjustment;

pub fn summarize_escalation_forecasts_for_policy(
    items: &[escalation_escalation_adjustment::EscalationForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_batched_timeline_policies_from_forecast(
    items: &[escalation_escalation_adjustment::EscalationForecast],
) -> Vec<BatchedTimelinePolicy> {
    let mut out: Vec<BatchedTimelinePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_timeline_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_batched_timeline_policy(item: &BatchedTimelinePolicy) -> String {
    escalation_escalation_adjustment::classify_escalation_forecast(item.measured_value())
}
