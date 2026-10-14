use super::*;

use std::collections::BTreeMap;

use crate::policy::digest as policy_digest;

pub fn summarize_policy_rollups_for_profile(items: &[policy_digest::PolicyRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_quote_profiles_from_rollup(items: &[policy_digest::PolicyRollup]) -> Vec<QuoteProfile> {
    let mut out: Vec<QuoteProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_profile(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_quote_profile(item: &QuoteProfile) -> String {
    policy_digest::classify_policy_rollup(item.projected_value())
}
