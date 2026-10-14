use super::*;

use std::collections::BTreeMap;

use crate::dns_record::ledger as dns_record_ledger;

pub fn summarize_lenient_dns_record_quotas_for_variance(
    items: &[dns_record_ledger::LenientDnsRecordQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_primary_health_check_variances_from_quota(
    items: &[dns_record_ledger::LenientDnsRecordQuota],
) -> Vec<PrimaryHealthCheckVariance> {
    let mut out: Vec<PrimaryHealthCheckVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_health_check_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_primary_health_check_variance(item: &PrimaryHealthCheckVariance) -> String {
    dns_record_ledger::classify_regional_dns_record_segment(item.projected_volume())
}
