use super::*;

use std::collections::BTreeMap;

use crate::retention::variance as retention_variance;

pub fn summarize_retention_variances_for_draft(
    items: &[retention_variance::RetentionVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_primary_sink_drafts_from_variance(
    items: &[retention_variance::RetentionVariance],
) -> Vec<PrimarySinkDraft> {
    let mut out: Vec<PrimarySinkDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_sink_draft(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_primary_sink_draft(item: &PrimarySinkDraft) -> String {
    retention_variance::classify_retention_variance(item.projected_exposure())
}
