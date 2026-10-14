use super::*;

use std::collections::BTreeMap;

use crate::allowlist::baseline as allowlist_baseline;

pub fn summarize_allowlist_baselines_for_draft(
    items: &[allowlist_baseline::AllowlistBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_tiered_allowlist_drafts_from_baseline(
    items: &[allowlist_baseline::AllowlistBaseline],
) -> Vec<TieredAllowlistDraft> {
    let mut out: Vec<TieredAllowlistDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_allowlist_draft(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_tiered_allowlist_draft(item: &TieredAllowlistDraft) -> String {
    allowlist_baseline::allowlist_baseline_bucket(item.projected_total())
}
