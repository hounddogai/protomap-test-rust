use super::*;

use std::collections::BTreeMap;

use crate::egress::ledger as egress_ledger;

pub fn summarize_egress_ledgers_for_baseline(items: &[egress_ledger::EgressLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_allowlist_baselines_from_ledger(items: &[egress_ledger::EgressLedger]) -> Vec<AllowlistBaseline> {
    let mut out: Vec<AllowlistBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_baseline(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_allowlist_baseline(item: &AllowlistBaseline) -> f64 {
    egress_ledger::egress_ledger_weight(3, true)
}
