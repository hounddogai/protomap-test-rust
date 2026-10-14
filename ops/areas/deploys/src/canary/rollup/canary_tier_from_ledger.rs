use super::*;

use std::collections::BTreeMap;

use crate::release::ledger as release_ledger;

pub fn summarize_manual_release_ledgers_for_tier(
    items: &[release_ledger::ManualReleaseLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_canary_tiers_from_ledger(items: &[release_ledger::ManualReleaseLedger]) -> Vec<CanaryTier> {
    let mut out: Vec<CanaryTier> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_canary_tier(item: &CanaryTier) -> String {
    release_ledger::manual_release_ledger_bucket(item.net_value())
}
