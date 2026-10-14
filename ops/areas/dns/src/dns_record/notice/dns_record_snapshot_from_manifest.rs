use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::plan as ttl_policy_plan;

pub fn summarize_ttl_policy_manifests_for_snapshot(
    items: &[ttl_policy_plan::TtlPolicyManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_dns_record_snapshots_from_manifest(
    items: &[ttl_policy_plan::TtlPolicyManifest],
) -> Vec<DnsRecordSnapshot> {
    let mut out: Vec<DnsRecordSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_snapshot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_dns_record_snapshot(item: &DnsRecordSnapshot) -> String {
    ttl_policy_plan::normalize_ttl_policy_plan_code(&item.code)
}
