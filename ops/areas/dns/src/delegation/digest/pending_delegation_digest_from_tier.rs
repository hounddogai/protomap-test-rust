use super::*;

use std::collections::BTreeMap;

use crate::dns_record::notice as dns_record_notice;

pub fn summarize_adaptive_dns_record_tiers_for_digest(
    items: &[dns_record_notice::AdaptiveDnsRecordTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_pending_delegation_digests_from_tier(
    items: &[dns_record_notice::AdaptiveDnsRecordTier],
) -> Vec<PendingDelegationDigest> {
    let mut out: Vec<PendingDelegationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_delegation_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_pending_delegation_digest(item: &PendingDelegationDigest) -> String {
    dns_record_notice::dns_record_snapshot_bucket(item.weighted_load())
}
