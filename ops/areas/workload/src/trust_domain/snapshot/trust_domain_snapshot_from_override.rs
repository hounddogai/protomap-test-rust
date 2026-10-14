use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::agreement as trust_domain_agreement;

pub fn summarize_primary_trust_domain_overrides_for_snapshot(
    items: &[trust_domain_agreement::PrimaryTrustDomainOverride],
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

pub fn derive_trust_domain_snapshots_from_override(
    items: &[trust_domain_agreement::PrimaryTrustDomainOverride],
) -> Vec<TrustDomainSnapshot> {
    let mut out: Vec<TrustDomainSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_snapshot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_trust_domain_snapshot(item: &TrustDomainSnapshot) -> String {
    trust_domain_agreement::classify_trust_domain_agreement(item.projected_value())
}
