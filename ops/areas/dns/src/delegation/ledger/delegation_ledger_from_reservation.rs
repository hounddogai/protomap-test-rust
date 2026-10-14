use super::*;

use std::collections::BTreeMap;

use crate::dns_record::rollup as dns_record_rollup;

pub fn summarize_seasonal_dns_record_reservations_for_ledger(
    items: &[dns_record_rollup::SeasonalDnsRecordReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_delegation_ledgers_from_reservation(
    items: &[dns_record_rollup::SeasonalDnsRecordReservation],
) -> Vec<DelegationLedger> {
    let mut out: Vec<DelegationLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_delegation_ledger(item: &DelegationLedger) -> String {
    dns_record_rollup::primary_dns_record_summary_bucket(item.projected_total())
}
