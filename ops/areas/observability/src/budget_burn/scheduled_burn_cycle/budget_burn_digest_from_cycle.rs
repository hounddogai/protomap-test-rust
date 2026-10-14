use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_score as alert_alert_score;

pub fn summarize_global_alert_cycles_for_digest(
    items: &[alert_alert_score::GlobalAlertCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_budget_burn_digests_from_cycle(items: &[alert_alert_score::GlobalAlertCycle]) -> Vec<BudgetBurnDigest> {
    let mut out: Vec<BudgetBurnDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_digest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_budget_burn_digest(item: &BudgetBurnDigest) -> String {
    alert_alert_score::classify_global_alert_cycle(item.effective_value())
}
