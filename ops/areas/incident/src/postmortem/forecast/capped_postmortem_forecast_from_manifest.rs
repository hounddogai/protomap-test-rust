use super::*;

use std::collections::BTreeMap;

use crate::escalation::variance as escalation_variance;

pub fn summarize_escalation_manifests_for_forecast(
    items: &[escalation_variance::EscalationManifest],
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

pub fn derive_capped_postmortem_forecasts_from_manifest(
    items: &[escalation_variance::EscalationManifest],
) -> Vec<CappedPostmortemForecast> {
    let mut out: Vec<CappedPostmortemForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_postmortem_forecast(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_capped_postmortem_forecast(item: &CappedPostmortemForecast) -> String {
    escalation_variance::escalation_manifest_bucket(item.projected_volume())
}
