use super::*;

use std::collections::BTreeMap;

use crate::federation::group as federation_group;

pub fn summarize_federation_forecasts_for_bundle(
    items: &[federation_group::FederationForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_binding_bundles_from_forecast(items: &[federation_group::FederationForecast]) -> Vec<BindingBundle> {
    let mut out: Vec<BindingBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_bundle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_binding_bundle(item: &BindingBundle) -> String {
    federation_group::normalize_federation_template_code(&item.code)
}
