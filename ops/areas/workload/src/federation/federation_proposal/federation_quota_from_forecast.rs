use super::*;

use std::collections::BTreeMap;

use crate::binding::template as binding_template;

pub fn summarize_binding_forecasts_for_quota(items: &[binding_template::BindingForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_federation_quotas_from_forecast(items: &[binding_template::BindingForecast]) -> Vec<FederationQuota> {
    let mut out: Vec<FederationQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_federation_quota(item: &FederationQuota) -> String {
    binding_template::lenient_binding_reservation_bucket(item.effective_volume())
}
