use super::*;

use std::collections::BTreeMap;

use crate::region_pin::adjustment as region_pin_adjustment;

pub fn summarize_region_pin_agreements_for_claim(
    items: &[region_pin_adjustment::RegionPinAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_global_isolation_claims_from_agreement(
    items: &[region_pin_adjustment::RegionPinAgreement],
) -> Vec<GlobalIsolationClaim> {
    let mut out: Vec<GlobalIsolationClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_global_isolation_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_global_isolation_claim(item: &GlobalIsolationClaim) -> String {
    region_pin_adjustment::normalize_capped_region_pin_manifest_code(&item.code)
}
