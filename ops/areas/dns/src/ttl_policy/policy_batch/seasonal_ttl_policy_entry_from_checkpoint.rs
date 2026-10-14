use super::*;

use std::collections::BTreeMap;

use crate::dns_record::record_checkpoint as dns_record_record_checkpoint;

pub fn summarize_seasonal_dns_record_checkpoints_for_entry(
    items: &[dns_record_record_checkpoint::SeasonalDnsRecordCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_seasonal_ttl_policy_entries_from_checkpoint(
    items: &[dns_record_record_checkpoint::SeasonalDnsRecordCheckpoint],
) -> Vec<SeasonalTtlPolicyEntry> {
    let mut out: Vec<SeasonalTtlPolicyEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_ttl_policy_entry(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_ttl_policy_entry(item: &SeasonalTtlPolicyEntry) -> String {
    dns_record_record_checkpoint::dns_record_proposal_bucket(item.weighted_volume())
}
