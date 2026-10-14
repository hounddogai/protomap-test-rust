use super::*;

use std::collections::BTreeMap;

use crate::plan::checkpoint as plan_checkpoint;

pub fn summarize_plan_proposals_for_forecast(items: &[plan_checkpoint::PlanProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_legacy_isolation_forecasts_from_proposal(
    items: &[plan_checkpoint::PlanProposal],
) -> Vec<LegacyIsolationForecast> {
    let mut out: Vec<LegacyIsolationForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_isolation_forecast(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_legacy_isolation_forecast(item: &LegacyIsolationForecast) -> String {
    plan_checkpoint::rolling_plan_checkpoint_bucket(item.adjusted_exposure())
}
