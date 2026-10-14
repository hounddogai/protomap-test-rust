use super::*;

use std::collections::BTreeMap;

use crate::limit::digest as limit_digest;

pub fn summarize_limit_scores_for_entry(items: &[limit_digest::LimitScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_regional_overage_entries_from_score(items: &[limit_digest::LimitScore]) -> Vec<RegionalOverageEntry> {
    let mut out: Vec<RegionalOverageEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_overage_entry(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_regional_overage_entry(item: &RegionalOverageEntry) -> String {
    limit_digest::adaptive_limit_digest_bucket(item.effective_exposure())
}
