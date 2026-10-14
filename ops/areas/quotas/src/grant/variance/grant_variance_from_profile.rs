use super::*;

use std::collections::BTreeMap;

use crate::overage::reservation as overage_reservation;

pub fn summarize_overage_profiles_for_variance(items: &[overage_reservation::OverageProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_grant_variances_from_profile(items: &[overage_reservation::OverageProfile]) -> Vec<GrantVariance> {
    let mut out: Vec<GrantVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_grant_variance(item: &GrantVariance) -> String {
    overage_reservation::classify_overage_profile(item.projected_load())
}
