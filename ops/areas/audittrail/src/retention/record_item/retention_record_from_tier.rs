use super::*;

use std::collections::BTreeMap;

use crate::redaction::tier as redaction_tier;

pub fn summarize_global_redaction_tiers_for_record(
    items: &[redaction_tier::GlobalRedactionTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_retention_records_from_tier(items: &[redaction_tier::GlobalRedactionTier]) -> Vec<RetentionRecord> {
    let mut out: Vec<RetentionRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_record(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_retention_record(item: &RetentionRecord) -> String {
    redaction_tier::global_redaction_tier_bucket(item.net_load())
}
