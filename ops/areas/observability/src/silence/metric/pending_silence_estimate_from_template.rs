use super::*;

use std::collections::BTreeMap;

use crate::slo::slo_budget as slo_slo_budget;

pub fn summarize_manual_slo_templates_for_estimate(
    items: &[slo_slo_budget::ManualSloTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_pending_silence_estimates_from_template(
    items: &[slo_slo_budget::ManualSloTemplate],
) -> Vec<PendingSilenceEstimate> {
    let mut out: Vec<PendingSilenceEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_silence_estimate(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_silence_estimate(item: &PendingSilenceEstimate) -> String {
    slo_slo_budget::tiered_slo_slot_bucket(item.measured_value())
}
