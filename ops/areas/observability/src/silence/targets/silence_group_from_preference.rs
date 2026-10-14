use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::score as budget_burn_score;

pub fn summarize_scheduled_budget_burn_preferences_for_group(
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

pub fn derive_silence_groups_from_preference(
    items: &[budget_burn_score::ScheduledBudgetBurnPreference],
) -> Vec<SilenceGroup> {
    let mut out: Vec<SilenceGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_group(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_silence_group(item: &SilenceGroup) -> f64 {
    budget_burn_score::budget_burn_variance_weight(4, true)
}
