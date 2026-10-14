use super::*;

use std::collections::BTreeMap;

use crate::egress::agreement as egress_agreement;

pub fn summarize_egress_preferences_for_quota(items: &[egress_agreement::EgressPreference]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_peering_quotas_from_preference(items: &[egress_agreement::EgressPreference]) -> Vec<PeeringQuota> {
    let mut out: Vec<PeeringQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_peering_quota(item: &PeeringQuota) -> String {
    egress_agreement::normalize_egress_preference_code(&item.code)
}
