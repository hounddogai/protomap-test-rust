use super::*;

use std::collections::BTreeMap;

use crate::overage::budget as overage_budget;

pub fn summarize_lenient_overage_forecasts_for_manifest(
    items: &[overage_budget::LenientOverageForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_lenient_grant_manifests_from_forecast(
    items: &[overage_budget::LenientOverageForecast],
) -> Vec<LenientGrantManifest> {
    let mut out: Vec<LenientGrantManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_grant_manifest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_lenient_grant_manifest(item: &LenientGrantManifest) -> String {
    overage_budget::strict_overage_rule_bucket(item.projected_total())
}
