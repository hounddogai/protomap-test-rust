use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_tier as isolation_isolation_tier;

pub fn summarize_regional_isolation_tiers_for_forecast(
    items: &[isolation_isolation_tier::RegionalIsolationTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_fallback_suspension_forecasts_from_tier(
    items: &[isolation_isolation_tier::RegionalIsolationTier],
) -> Vec<FallbackSuspensionForecast> {
    let mut out: Vec<FallbackSuspensionForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_suspension_forecast(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_fallback_suspension_forecast(item: &FallbackSuspensionForecast) -> String {
    isolation_isolation_tier::manual_isolation_proposal_bucket(item.effective_exposure())
}
