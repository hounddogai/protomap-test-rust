use super::*;

use std::collections::BTreeMap;

use crate::rollback::tier as rollback_tier;

pub fn summarize_rollback_tiers_for_forecast(items: &[rollback_tier::RollbackTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_legacy_rollout_forecasts_from_tier(items: &[rollback_tier::RollbackTier]) -> Vec<LegacyRolloutForecast> {
    let mut out: Vec<LegacyRolloutForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_rollout_forecast(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_legacy_rollout_forecast(item: &LegacyRolloutForecast) -> f64 {
    rollback_tier::legacy_rollback_milestone_weight(2, true)
}
