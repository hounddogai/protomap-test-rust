use super::*;

use std::collections::BTreeMap;

use crate::overage::slot as overage_slot;

pub fn summarize_overage_slots_for_summary(items: &[overage_slot::OverageSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_manual_forecast_summaries_from_slot(items: &[overage_slot::OverageSlot]) -> Vec<ManualForecastSummary> {
    let mut out: Vec<ManualForecastSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_forecast_summary(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_manual_forecast_summary(item: &ManualForecastSummary) -> String {
    overage_slot::normalize_overage_slot_code(&item.code)
}
