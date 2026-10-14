use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::assignment as trust_domain_assignment;

pub fn summarize_trust_domain_metrics_for_entry(
    items: &[trust_domain_assignment::TrustDomainMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_token_entries_from_metric(items: &[trust_domain_assignment::TrustDomainMetric]) -> Vec<TokenEntry> {
    let mut out: Vec<TokenEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_token_entry(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_token_entry(item: &TokenEntry) -> String {
    trust_domain_assignment::regional_trust_domain_notice_bucket(item.net_load())
}
