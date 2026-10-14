use super::*;

use std::collections::BTreeMap;

use crate::binding::group as binding_group;

pub fn summarize_binding_preferences_for_quota(items: &[binding_group::BindingPreference]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_token_quotas_from_preference(items: &[binding_group::BindingPreference]) -> Vec<TokenQuota> {
    let mut out: Vec<TokenQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_token_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_token_quota(item: &TokenQuota) -> f64 {
    binding_group::binding_group_weight(2, true)
}
