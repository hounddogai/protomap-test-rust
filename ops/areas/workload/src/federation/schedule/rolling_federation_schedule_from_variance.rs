use super::*;

use std::collections::BTreeMap;

use crate::identity::identity_forecast as identity_identity_forecast;

pub fn summarize_identity_variances_for_schedule(
    items: &[identity_identity_forecast::IdentityVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_rolling_federation_schedules_from_variance(
    items: &[identity_identity_forecast::IdentityVariance],
) -> Vec<RollingFederationSchedule> {
    let mut out: Vec<RollingFederationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_federation_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_rolling_federation_schedule(item: &RollingFederationSchedule) -> String {
    identity_identity_forecast::classify_identity_variance(item.weighted_exposure())
}
