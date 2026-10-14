use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::policy as budget_burn_policy;

pub fn summarize_budget_burn_policies_for_assignment(
    items: &[budget_burn_policy::BudgetBurnPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_adaptive_silence_assignments_from_policy(
    items: &[budget_burn_policy::BudgetBurnPolicy],
) -> Vec<AdaptiveSilenceAssignment> {
    let mut out: Vec<AdaptiveSilenceAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_silence_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_adaptive_silence_assignment(item: &AdaptiveSilenceAssignment) -> String {
    budget_burn_policy::classify_strict_budget_burn_window(item.projected_load())
}
