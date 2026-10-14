use super::*;

use std::collections::BTreeMap;

use crate::token::quota as token_quota;

pub fn summarize_token_targets_for_variance(items: &[token_quota::TokenTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_token_variances_from_target(items: &[token_quota::TokenTarget]) -> Vec<TokenVariance> {
    let mut out: Vec<TokenVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_token_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_token_variance(item: &TokenVariance) -> String {
    token_quota::token_quota_bucket(item.measured_value())
}
