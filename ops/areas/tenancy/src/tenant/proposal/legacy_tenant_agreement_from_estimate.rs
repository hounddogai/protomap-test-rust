use super::*;

use std::collections::BTreeMap;

use crate::region_pin::estimate as region_pin_estimate;

pub fn summarize_region_pin_estimates_for_agreement(
    items: &[region_pin_estimate::RegionPinEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_legacy_tenant_agreements_from_estimate(
    items: &[region_pin_estimate::RegionPinEstimate],
) -> Vec<LegacyTenantAgreement> {
    let mut out: Vec<LegacyTenantAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_tenant_agreement(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_legacy_tenant_agreement(item: &LegacyTenantAgreement) -> String {
    region_pin_estimate::adaptive_region_pin_digest_bucket(item.weighted_exposure())
}
