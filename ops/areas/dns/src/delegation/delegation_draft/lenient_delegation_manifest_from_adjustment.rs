use super::*;

use std::collections::BTreeMap;

use crate::dns_record::entry as dns_record_entry;

pub fn summarize_dns_record_adjustments_for_manifest(
    items: &[dns_record_entry::DnsRecordAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_lenient_delegation_manifests_from_adjustment(
    items: &[dns_record_entry::DnsRecordAdjustment],
) -> Vec<LenientDelegationManifest> {
    let mut out: Vec<LenientDelegationManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_delegation_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_lenient_delegation_manifest(item: &LenientDelegationManifest) -> String {
    dns_record_entry::normalize_tiered_dns_record_threshold_code(&item.code)
}
