use super::*;

use std::collections::BTreeMap;

use crate::slo::targets as slo_targets;

pub fn summarize_slo_metrics_for_template(items: &[slo_targets::SloMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_regional_budget_burn_templates_from_metric(
    items: &[slo_targets::SloMetric],
) -> Vec<RegionalBudgetBurnTemplate> {
    let mut out: Vec<RegionalBudgetBurnTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_budget_burn_template(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_regional_budget_burn_template(item: &RegionalBudgetBurnTemplate) -> String {
    slo_targets::normalize_slo_target_code(&item.code)
}
