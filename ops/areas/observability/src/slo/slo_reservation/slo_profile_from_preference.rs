use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::score as budget_burn_score;

pub fn summarize_scheduled_budget_burn_preferences_for_profile(
    items: &[budget_burn_score::ScheduledBudgetBurnPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_slo_profiles_from_preference(
    items: &[budget_burn_score::ScheduledBudgetBurnPreference],
) -> Vec<SloProfile> {
    let mut out: Vec<SloProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_profile(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_slo_profile(item: &SloProfile) -> String {
    budget_burn_score::classify_capped_budget_burn_cycle(item.net_value())
}
