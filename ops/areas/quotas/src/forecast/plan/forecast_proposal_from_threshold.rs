use super::*;

use std::collections::BTreeMap;

use crate::grant::manifest as grant_manifest;

pub fn summarize_regional_grant_thresholds_for_proposal(
    items: &[grant_manifest::RegionalGrantThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_forecast_proposals_from_threshold(
    items: &[grant_manifest::RegionalGrantThreshold],
) -> Vec<ForecastProposal> {
    let mut out: Vec<ForecastProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_proposal(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_forecast_proposal(item: &ForecastProposal) -> String {
    grant_manifest::lenient_grant_variance_bucket(item.measured_value())
}
