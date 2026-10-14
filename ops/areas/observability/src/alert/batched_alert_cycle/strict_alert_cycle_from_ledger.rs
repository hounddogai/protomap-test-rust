use super::*;

use std::collections::BTreeMap;

use crate::probe::ledger as probe_ledger;

pub fn summarize_probe_ledgers_for_cycle(items: &[probe_ledger::ProbeLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_strict_alert_cycles_from_ledger(items: &[probe_ledger::ProbeLedger]) -> Vec<StrictAlertCycle> {
    let mut out: Vec<StrictAlertCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_alert_cycle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_strict_alert_cycle(item: &StrictAlertCycle) -> String {
    probe_ledger::probe_ledger_bucket(item.adjusted_load())
}
