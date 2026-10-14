use super::*;

use std::collections::BTreeMap;

use crate::grant::grant_snapshot as grant_grant_snapshot;

pub fn summarize_grant_manifests_for_estimate(items: &[grant_grant_snapshot::GrantManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_seasonal_overage_estimates_from_manifest(
    items: &[grant_grant_snapshot::GrantManifest],
) -> Vec<SeasonalOverageEstimate> {
    let mut out: Vec<SeasonalOverageEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_overage_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_seasonal_overage_estimate(item: &SeasonalOverageEstimate) -> f64 {
    grant_grant_snapshot::grant_manifest_weight(7, true)
}
