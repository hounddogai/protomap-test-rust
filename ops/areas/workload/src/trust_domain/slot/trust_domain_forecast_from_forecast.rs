use super::*;

use std::collections::BTreeMap;

use crate::identity::identity_forecast as identity_identity_forecast;

pub fn summarize_default_identity_forecasts_for_forecast(
    items: &[identity_identity_forecast::DefaultIdentityForecast],
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

pub fn derive_trust_domain_forecasts_from_forecast(
    items: &[identity_identity_forecast::DefaultIdentityForecast],
) -> Vec<TrustDomainForecast> {
    let mut out: Vec<TrustDomainForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_trust_domain_forecast(item: &TrustDomainForecast) -> String {
    identity_identity_forecast::classify_identity_variance(item.net_exposure())
}
