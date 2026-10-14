use super::*;

use std::collections::BTreeMap;

use crate::binding::rollup as binding_rollup;

pub fn summarize_binding_agreements_for_digest(items: &[binding_rollup::BindingAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_rolling_token_digests_from_agreement(
    items: &[binding_rollup::BindingAgreement],
) -> Vec<RollingTokenDigest> {
    let mut out: Vec<RollingTokenDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_token_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_rolling_token_digest(item: &RollingTokenDigest) -> f64 {
    binding_rollup::binding_agreement_weight(6, true)
}
