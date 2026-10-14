use super::*;

use std::collections::BTreeMap;

use crate::dns_record::ledger as dns_record_ledger;

pub fn summarize_capped_dns_record_entries_for_cycle(
    items: &[dns_record_ledger::CappedDnsRecordEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_legacy_failover_cycles_from_entry(
    items: &[dns_record_ledger::CappedDnsRecordEntry],
) -> Vec<LegacyFailoverCycle> {
    let mut out: Vec<LegacyFailoverCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_failover_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_legacy_failover_cycle(item: &LegacyFailoverCycle) -> String {
    dns_record_ledger::dns_record_ledger_bucket(item.net_exposure())
}
