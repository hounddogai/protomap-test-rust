use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::policy as budget_burn_policy;

pub fn summarize_strict_budget_burn_windows_for_audit(
    items: &[budget_burn_policy::StrictBudgetBurnWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_alert_audits_from_window(items: &[budget_burn_policy::StrictBudgetBurnWindow]) -> Vec<AlertAudit> {
    let mut out: Vec<AlertAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_audit(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_alert_audit(item: &AlertAudit) -> String {
    budget_burn_policy::normalize_budget_burn_policy_code(&item.code)
}
