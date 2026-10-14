use super::*;

use std::collections::BTreeMap;

use crate::health_check::batch as health_check_batch;

pub fn summarize_lenient_health_check_summaries_for_rollup(
    items: &[health_check_batch::LenientHealthCheckSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_dns_record_rollups_from_summary(
    items: &[health_check_batch::LenientHealthCheckSummary],
) -> Vec<DnsRecordRollup> {
    let mut out: Vec<DnsRecordRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_rollup(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_dns_record_rollup(item: &DnsRecordRollup) -> f64 {
    health_check_batch::lenient_health_check_summary_weight(6, true)
}
