use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_budget as verifier_verifier_budget;

pub fn summarize_pending_verifier_assignments_for_forecast(
    items: &[verifier_verifier_budget::PendingVerifierAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_rolling_verifier_forecasts_from_assignment(
    items: &[verifier_verifier_budget::PendingVerifierAssignment],
) -> Vec<RollingVerifierForecast> {
    let mut out: Vec<RollingVerifierForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_verifier_forecast(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rolling_verifier_forecast(item: &RollingVerifierForecast) -> f64 {
    verifier_verifier_budget::pending_verifier_budget_weight(6, true)
}
