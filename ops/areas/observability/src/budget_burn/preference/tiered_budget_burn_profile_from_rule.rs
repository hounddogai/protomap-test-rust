use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_rule as alert_alert_rule;

pub fn summarize_fallback_alert_rules_for_profile(
    items: &[alert_alert_rule::FallbackAlertRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_tiered_budget_burn_profiles_from_rule(
    items: &[alert_alert_rule::FallbackAlertRule],
) -> Vec<TieredBudgetBurnProfile> {
    let mut out: Vec<TieredBudgetBurnProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_budget_burn_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_tiered_budget_burn_profile(item: &TieredBudgetBurnProfile) -> String {
    alert_alert_rule::classify_alert_ledger(item.adjusted_total())
}
