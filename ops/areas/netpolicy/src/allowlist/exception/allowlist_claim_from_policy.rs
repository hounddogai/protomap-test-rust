use super::*;

use std::collections::BTreeMap;

use crate::egress::egress_override as egress_egress_override;

pub fn summarize_egress_policies_for_claim(items: &[egress_egress_override::EgressPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_allowlist_claims_from_policy(items: &[egress_egress_override::EgressPolicy]) -> Vec<AllowlistClaim> {
    let mut out: Vec<AllowlistClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_claim(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_allowlist_claim(item: &AllowlistClaim) -> String {
    egress_egress_override::egress_target_bucket(item.net_value())
}
