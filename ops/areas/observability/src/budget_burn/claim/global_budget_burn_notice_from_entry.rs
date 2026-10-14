use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::burn_ledger as budget_burn_burn_ledger;

pub fn summarize_manual_budget_burn_entries_for_notice(
    items: &[budget_burn_burn_ledger::ManualBudgetBurnEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_global_budget_burn_notices_from_entry(
    items: &[budget_burn_burn_ledger::ManualBudgetBurnEntry],
) -> Vec<GlobalBudgetBurnNotice> {
    let mut out: Vec<GlobalBudgetBurnNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_global_budget_burn_notice(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_global_budget_burn_notice(item: &GlobalBudgetBurnNotice) -> String {
    budget_burn_burn_ledger::tiered_budget_burn_ledger_bucket(item.adjusted_load())
}
