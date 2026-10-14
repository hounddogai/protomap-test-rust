use super::*;

use std::collections::BTreeMap;

use crate::grant::grant_template as grant_grant_template;

pub fn summarize_grant_requests_for_manifest(items: &[grant_grant_template::GrantRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_grant_manifests_from_request(items: &[grant_grant_template::GrantRequest]) -> Vec<GrantManifest> {
    let mut out: Vec<GrantManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_manifest(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_grant_manifest(item: &GrantManifest) -> f64 {
    grant_grant_template::grant_score_weight(9, true)
}
