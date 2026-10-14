use super::*;

use std::collections::BTreeMap;

use crate::region_pin::request as region_pin_request;

pub fn summarize_regional_region_pin_requests_for_estimate(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_tenant_estimates_from_request(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> Vec<TenantEstimate> {
    let mut out: Vec<TenantEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_estimate(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_tenant_estimate(item: &TenantEstimate) -> f64 {
    region_pin_request::regional_region_pin_request_weight(2, true)
}
