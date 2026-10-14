use super::*;

use std::collections::BTreeMap;

use crate::delegation::legacy_delegation_score as delegation_legacy_delegation_score;

pub fn summarize_scheduled_delegation_scores_for_variance(
    items: &[delegation_legacy_delegation_score::ScheduledDelegationScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_default_dns_record_variances_from_score(
    items: &[delegation_legacy_delegation_score::ScheduledDelegationScore],
) -> Vec<DefaultDnsRecordVariance> {
    let mut out: Vec<DefaultDnsRecordVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_default_dns_record_variance(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_default_dns_record_variance(item: &DefaultDnsRecordVariance) -> f64 {
    delegation_legacy_delegation_score::adaptive_delegation_request_weight(8, true)
}
