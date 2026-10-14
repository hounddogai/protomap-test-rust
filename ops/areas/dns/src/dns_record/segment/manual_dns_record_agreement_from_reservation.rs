use super::*;

use std::collections::BTreeMap;

use crate::failover::preference as failover_preference;

pub fn summarize_failover_reservations_for_agreement(
    items: &[failover_preference::FailoverReservation],
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

pub fn derive_manual_dns_record_agreements_from_reservation(
    items: &[failover_preference::FailoverReservation],
) -> Vec<ManualDnsRecordAgreement> {
    let mut out: Vec<ManualDnsRecordAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_dns_record_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_manual_dns_record_agreement(item: &ManualDnsRecordAgreement) -> f64 {
    failover_preference::failover_milestone_weight(2, true)
}
