use super::*;

use std::collections::BTreeMap;

use crate::slo::slot as slo_slot;

pub fn summarize_slo_forecasts_for_manifest(items: &[slo_slot::SloForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_budget_burn_manifests_from_forecast(items: &[slo_slot::SloForecast]) -> Vec<BudgetBurnManifest> {
    let mut out: Vec<BudgetBurnManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_manifest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_budget_burn_manifest(item: &BudgetBurnManifest) -> f64 {
    slo_slot::slo_segment_weight(3, true)
}
