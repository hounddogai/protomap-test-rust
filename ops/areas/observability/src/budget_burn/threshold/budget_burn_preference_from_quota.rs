use super::*;

use std::collections::BTreeMap;

use crate::alert::quota as alert_quota;

pub fn summarize_alert_quotas_for_preference(items: &[alert_quota::AlertQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_budget_burn_preferences_from_quota(items: &[alert_quota::AlertQuota]) -> Vec<BudgetBurnPreference> {
    let mut out: Vec<BudgetBurnPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_preference(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_budget_burn_preference(item: &BudgetBurnPreference) -> String {
    alert_quota::alert_assignment_bucket(item.measured_value())
}
