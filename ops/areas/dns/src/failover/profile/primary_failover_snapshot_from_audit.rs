use super::*;

use std::collections::BTreeMap;

use crate::dns_record::audit as dns_record_audit;

pub fn summarize_dns_record_audits_for_snapshot(items: &[dns_record_audit::DnsRecordAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_primary_failover_snapshots_from_audit(
    items: &[dns_record_audit::DnsRecordAudit],
) -> Vec<PrimaryFailoverSnapshot> {
    let mut out: Vec<PrimaryFailoverSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_failover_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_primary_failover_snapshot(item: &PrimaryFailoverSnapshot) -> String {
    dns_record_audit::normalize_dns_record_review_code(&item.code)
}
