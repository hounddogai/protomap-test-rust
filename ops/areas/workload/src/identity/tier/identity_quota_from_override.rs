use super::*;

use std::collections::BTreeMap;

use crate::token::quota as token_quota;

pub fn summarize_token_overrides_for_quota(items: &[token_quota::TokenOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_identity_quotas_from_override(items: &[token_quota::TokenOverride]) -> Vec<IdentityQuota> {
    let mut out: Vec<IdentityQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_quota(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_identity_quota(item: &IdentityQuota) -> String {
    token_quota::token_quota_bucket(item.measured_value())
}
