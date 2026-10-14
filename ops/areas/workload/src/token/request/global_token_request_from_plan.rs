use super::*;

use std::collections::BTreeMap;

use crate::token::token_batch as token_token_batch;

pub fn summarize_strict_token_plans_for_request(items: &[token_token_batch::StrictTokenPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_global_token_requests_from_plan(items: &[token_token_batch::StrictTokenPlan]) -> Vec<GlobalTokenRequest> {
    let mut out: Vec<GlobalTokenRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_token_request(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_global_token_request(item: &GlobalTokenRequest) -> String {
    token_token_batch::normalize_regional_token_batch_code(&item.code)
}
