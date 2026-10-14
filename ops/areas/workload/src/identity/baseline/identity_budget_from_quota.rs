use super::*;

use std::collections::BTreeMap;

use crate::token::token_checkpoint as token_token_checkpoint;

pub fn summarize_seasonal_token_quotas_for_budget(
    items: &[token_token_checkpoint::SeasonalTokenQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_identity_budgets_from_quota(items: &[token_token_checkpoint::SeasonalTokenQuota]) -> Vec<IdentityBudget> {
    let mut out: Vec<IdentityBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_identity_budget(item: &IdentityBudget) -> String {
    token_token_checkpoint::classify_capped_token_checkpoint(item.adjusted_volume())
}
