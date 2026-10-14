use super::*;

use std::collections::BTreeMap;

use crate::identity::agreement as identity_agreement;

pub fn summarize_global_identity_manifests_for_baseline(
    items: &[identity_agreement::GlobalIdentityManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_seasonal_token_baselines_from_manifest(
    items: &[identity_agreement::GlobalIdentityManifest],
) -> Vec<SeasonalTokenBaseline> {
    let mut out: Vec<SeasonalTokenBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_token_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_seasonal_token_baseline(item: &SeasonalTokenBaseline) -> String {
    identity_agreement::classify_identity_agreement(item.effective_volume())
}
