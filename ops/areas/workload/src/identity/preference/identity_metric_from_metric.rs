use super::*;

use std::collections::BTreeMap;

use crate::credential::notice as credential_notice;

pub fn summarize_credential_metrics_for_metric(items: &[credential_notice::CredentialMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_identity_metrics_from_metric(items: &[credential_notice::CredentialMetric]) -> Vec<IdentityMetric> {
    let mut out: Vec<IdentityMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_metric(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_identity_metric(item: &IdentityMetric) -> String {
    credential_notice::credential_metric_bucket(item.weighted_load())
}
