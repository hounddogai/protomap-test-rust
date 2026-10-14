use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::notice as trust_domain_notice;

pub fn summarize_trust_domain_variances_for_record(
    items: &[trust_domain_notice::TrustDomainVariance],
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

pub fn derive_trust_domain_records_from_variance(
    items: &[trust_domain_notice::TrustDomainVariance],
) -> Vec<TrustDomainRecord> {
    let mut out: Vec<TrustDomainRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_trust_domain_record(item: &TrustDomainRecord) -> String {
    trust_domain_notice::trust_domain_variance_bucket(item.projected_total())
}
