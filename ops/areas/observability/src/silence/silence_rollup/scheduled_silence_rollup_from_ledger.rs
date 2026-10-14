use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::ledger as budget_burn_ledger;

pub fn summarize_budget_burn_ledgers_for_rollup(
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

pub fn derive_scheduled_silence_rollups_from_ledger(
    items: &[budget_burn_ledger::BudgetBurnLedger],
) -> Vec<ScheduledSilenceRollup> {
    let mut out: Vec<ScheduledSilenceRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_silence_rollup(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_scheduled_silence_rollup_extra(item: &ScheduledSilenceRollup) -> String {
    budget_burn_ledger::budget_burn_audit_bucket(item.projected_total())
}
