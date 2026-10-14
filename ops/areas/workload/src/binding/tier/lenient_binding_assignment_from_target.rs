use super::*;

use std::collections::BTreeMap;

use crate::token::quota as token_quota;

pub fn summarize_token_targets_for_assignment(items: &[token_quota::TokenTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_lenient_binding_assignments_from_target(
    items: &[token_quota::TokenTarget],
) -> Vec<LenientBindingAssignment> {
    let mut out: Vec<LenientBindingAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_binding_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_lenient_binding_assignment(item: &LenientBindingAssignment) -> String {
    token_quota::classify_token_quota(item.effective_load())
}
