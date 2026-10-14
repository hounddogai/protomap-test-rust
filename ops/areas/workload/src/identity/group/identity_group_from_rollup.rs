use super::*;

use std::collections::BTreeMap;

use crate::token::token_baseline as token_token_baseline;

pub fn summarize_token_rollups_for_group(items: &[token_token_baseline::TokenRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_identity_groups_from_rollup(items: &[token_token_baseline::TokenRollup]) -> Vec<IdentityGroup> {
    let mut out: Vec<IdentityGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_group(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_identity_group(item: &IdentityGroup) -> String {
    token_token_baseline::normalize_seasonal_token_baseline_code(&item.code)
}
