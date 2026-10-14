use super::*;

use std::collections::BTreeMap;

use crate::health_check::rollup as health_check_rollup;

pub fn summarize_legacy_health_check_quotas_for_draft(
    items: &[health_check_rollup::LegacyHealthCheckQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_dns_record_drafts_from_quota(
    items: &[health_check_rollup::LegacyHealthCheckQuota],
) -> Vec<DnsRecordDraft> {
    let mut out: Vec<DnsRecordDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_draft(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_dns_record_draft(item: &DnsRecordDraft) -> String {
    health_check_rollup::classify_health_check_checkpoint(item.weighted_exposure())
}
