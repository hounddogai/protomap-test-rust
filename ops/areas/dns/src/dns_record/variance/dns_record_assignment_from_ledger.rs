use super::*;

use std::collections::BTreeMap;

use crate::health_check::ledger as health_check_ledger;

pub fn summarize_tiered_health_check_ledgers_for_assignment(
    items: &[health_check_ledger::TieredHealthCheckLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_dns_record_assignments_from_ledger(
    items: &[health_check_ledger::TieredHealthCheckLedger],
) -> Vec<DnsRecordAssignment> {
    let mut out: Vec<DnsRecordAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_assignment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_dns_record_assignment(item: &DnsRecordAssignment) -> String {
    health_check_ledger::classify_tiered_health_check_ledger(item.measured_value())
}
