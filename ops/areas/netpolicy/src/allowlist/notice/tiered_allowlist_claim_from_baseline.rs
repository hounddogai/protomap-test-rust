use super::*;

use std::collections::BTreeMap;

use crate::egress::egress_agreement as egress_egress_agreement;

pub fn summarize_egress_baselines_for_claim(
    items: &[egress_egress_agreement::EgressBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_tiered_allowlist_claims_from_baseline(
    items: &[egress_egress_agreement::EgressBaseline],
) -> Vec<TieredAllowlistClaim> {
    let mut out: Vec<TieredAllowlistClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_allowlist_claim(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_tiered_allowlist_claim_extra(item: &TieredAllowlistClaim) -> f64 {
    egress_egress_agreement::egress_baseline_weight(7, true)
}
