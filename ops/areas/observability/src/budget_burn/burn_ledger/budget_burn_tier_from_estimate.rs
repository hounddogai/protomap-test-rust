use super::*;

use std::collections::BTreeMap;

use crate::probe::estimate as probe_estimate;

pub fn summarize_strict_probe_estimates_for_tier(
    items: &[probe_estimate::StrictProbeEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_budget_burn_tiers_from_estimate(items: &[probe_estimate::StrictProbeEstimate]) -> Vec<BudgetBurnTier> {
    let mut out: Vec<BudgetBurnTier> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_budget_burn_tier(item: &BudgetBurnTier) -> f64 {
    probe_estimate::seasonal_probe_draft_weight(4, true)
}
