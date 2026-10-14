use super::*;

use std::collections::BTreeMap;

use crate::binding::rollup as binding_rollup;

pub fn summarize_binding_baselines_for_digest(items: &[binding_rollup::BindingBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_identity_digests_from_baseline(items: &[binding_rollup::BindingBaseline]) -> Vec<IdentityDigest> {
    let mut out: Vec<IdentityDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_digest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_identity_digest(item: &IdentityDigest) -> String {
    binding_rollup::classify_binding_rollup(item.effective_exposure())
}
