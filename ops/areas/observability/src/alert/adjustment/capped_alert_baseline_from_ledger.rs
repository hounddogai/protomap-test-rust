use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::ledger as budget_burn_ledger;

pub fn summarize_budget_burn_ledgers_for_baseline(
    items: &[budget_burn_ledger::BudgetBurnLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_capped_alert_baselines_from_ledger(
    items: &[budget_burn_ledger::BudgetBurnLedger],
) -> Vec<CappedAlertBaseline> {
    let mut out: Vec<CappedAlertBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_alert_baseline(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_capped_alert_baseline(item: &CappedAlertBaseline) -> String {
    budget_burn_ledger::classify_budget_burn_audit(item.net_value())
}
