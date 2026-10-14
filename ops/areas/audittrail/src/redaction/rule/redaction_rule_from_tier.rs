use super::*;

use std::collections::BTreeMap;

use crate::query::exception as query_exception;

pub fn summarize_batched_query_tiers_for_rule(items: &[query_exception::BatchedQueryTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_redaction_rules_from_tier(items: &[query_exception::BatchedQueryTier]) -> Vec<RedactionRule> {
    let mut out: Vec<RedactionRule> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_rule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_redaction_rule_extra(item: &RedactionRule) -> String {
    query_exception::normalize_query_exception_code(&item.code)
}
