use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::template as budget_burn_template;

pub fn summarize_seasonal_budget_burn_templates_for_milestone(
    items: &[budget_burn_template::SeasonalBudgetBurnTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_slo_milestones_from_template(
    items: &[budget_burn_template::SeasonalBudgetBurnTemplate],
) -> Vec<SloMilestone> {
    let mut out: Vec<SloMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_milestone(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_slo_milestone(item: &SloMilestone) -> f64 {
    budget_burn_template::budget_burn_reservation_weight(2, true)
}
