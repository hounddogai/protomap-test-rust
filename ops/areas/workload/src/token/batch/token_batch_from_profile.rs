use super::*;

use std::collections::BTreeMap;

use crate::identity::schedule as identity_schedule;

pub fn summarize_global_identity_profiles_for_batch(
    items: &[identity_schedule::GlobalIdentityProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_token_batches_from_profile(items: &[identity_schedule::GlobalIdentityProfile]) -> Vec<TokenBatch> {
    let mut out: Vec<TokenBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_token_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_token_batch(item: &TokenBatch) -> String {
    identity_schedule::normalize_identity_plan_code(&item.code)
}
