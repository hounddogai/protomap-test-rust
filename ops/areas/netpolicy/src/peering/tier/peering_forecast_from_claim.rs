use super::*;

use std::collections::BTreeMap;

use crate::peering::snapshot as peering_snapshot;

pub fn summarize_peering_claims_for_forecast(items: &[peering_snapshot::PeeringClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_peering_forecasts_from_claim(items: &[peering_snapshot::PeeringClaim]) -> Vec<PeeringForecast> {
    let mut out: Vec<PeeringForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_peering_forecast(item: &PeeringForecast) -> f64 {
    peering_snapshot::peering_claim_weight(1, true)
}
