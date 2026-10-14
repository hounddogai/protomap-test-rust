use super::*;

use std::collections::BTreeMap;

use crate::credential::forecast as credential_forecast;

pub fn summarize_default_credential_metrics_for_profile(
    items: &[credential_forecast::DefaultCredentialMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_token_profiles_from_metric(items: &[credential_forecast::DefaultCredentialMetric]) -> Vec<TokenProfile> {
    let mut out: Vec<TokenProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_token_profile(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_token_profile(item: &TokenProfile) -> String {
    credential_forecast::normalize_default_credential_metric_code(&item.code)
}
