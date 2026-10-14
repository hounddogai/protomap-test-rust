use super::*;

use std::collections::BTreeMap;

use crate::probe::slot as probe_slot;

pub fn summarize_probe_tiers_for_template(items: &[probe_slot::ProbeTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_rolling_budget_burn_templates_from_tier(
    items: &[probe_slot::ProbeTier],
) -> Vec<RollingBudgetBurnTemplate> {
    let mut out: Vec<RollingBudgetBurnTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_budget_burn_template(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_rolling_budget_burn_template(item: &RollingBudgetBurnTemplate) -> String {
    probe_slot::probe_milestone_bucket(item.weighted_value())
}
