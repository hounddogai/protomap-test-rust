use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::threshold as trust_domain_threshold;

pub fn summarize_trust_domain_thresholds_for_snapshot(
    items: &[trust_domain_threshold::TrustDomainThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_credential_snapshots_from_threshold(
    items: &[trust_domain_threshold::TrustDomainThreshold],
) -> Vec<CredentialSnapshot> {
    let mut out: Vec<CredentialSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_snapshot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_credential_snapshot(item: &CredentialSnapshot) -> f64 {
    trust_domain_threshold::trust_domain_threshold_weight(3, true)
}
