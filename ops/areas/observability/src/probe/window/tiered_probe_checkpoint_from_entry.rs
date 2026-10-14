use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::burn_ledger as budget_burn_burn_ledger;

pub fn summarize_manual_budget_burn_entries_for_checkpoint(
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

pub fn derive_tiered_probe_checkpoints_from_entry(
    items: &[budget_burn_burn_ledger::ManualBudgetBurnEntry],
) -> Vec<TieredProbeCheckpoint> {
    let mut out: Vec<TieredProbeCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_probe_checkpoint(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_tiered_probe_checkpoint(item: &TieredProbeCheckpoint) -> String {
    budget_burn_burn_ledger::manual_budget_burn_entry_bucket(item.weighted_total())
}
