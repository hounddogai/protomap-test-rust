use super::*;

use std::collections::BTreeMap;

use crate::forecast::audit as forecast_audit;

pub fn summarize_primary_forecast_mappings_for_budget(
    items: &[forecast_audit::PrimaryForecastMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_fallback_overage_budgets_from_mapping(
    items: &[forecast_audit::PrimaryForecastMapping],
) -> Vec<FallbackOverageBudget> {
    let mut out: Vec<FallbackOverageBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_overage_budget(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_fallback_overage_budget(item: &FallbackOverageBudget) -> String {
    forecast_audit::normalize_manual_forecast_audit_code(&item.code)
}
