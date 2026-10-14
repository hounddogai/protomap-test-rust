use super::*;

use std::collections::BTreeMap;

use crate::failover::entry as failover_entry;

pub fn summarize_failover_entries_for_agreement(items: &[failover_entry::FailoverEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_regional_failover_agreements_from_entry(
    items: &[failover_entry::FailoverEntry],
) -> Vec<RegionalFailoverAgreement> {
    let mut out: Vec<RegionalFailoverAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_failover_agreement(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_regional_failover_agreement(item: &RegionalFailoverAgreement) -> String {
    failover_entry::classify_regional_failover_checkpoint(item.adjusted_exposure())
}
