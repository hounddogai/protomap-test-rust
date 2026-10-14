use super::*;

use std::collections::BTreeMap;

use crate::slo::tier as slo_tier;

pub fn summarize_slo_ledgers_for_notice(items: &[slo_tier::SloLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_probe_notices_from_ledger(items: &[slo_tier::SloLedger]) -> Vec<ProbeNotice> {
    let mut out: Vec<ProbeNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_probe_notice(item: &ProbeNotice) -> String {
    slo_tier::slo_ledger_bucket(item.effective_total())
}
