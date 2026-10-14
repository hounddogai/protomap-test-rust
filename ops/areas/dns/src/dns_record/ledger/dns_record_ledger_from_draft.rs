use super::*;

use std::collections::BTreeMap;

use crate::delegation::estimate as delegation_estimate;

pub fn summarize_global_delegation_drafts_for_ledger(
    items: &[delegation_estimate::GlobalDelegationDraft],
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

pub fn derive_dns_record_ledgers_from_draft(
    items: &[delegation_estimate::GlobalDelegationDraft],
) -> Vec<DnsRecordLedger> {
    let mut out: Vec<DnsRecordLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_ledger(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_dns_record_ledger(item: &DnsRecordLedger) -> f64 {
    delegation_estimate::regional_delegation_estimate_weight(6, true)
}
