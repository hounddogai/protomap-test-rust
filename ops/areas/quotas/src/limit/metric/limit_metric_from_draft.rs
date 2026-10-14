use super::*;

use std::collections::BTreeMap;

use crate::grant::draft as grant_draft;

pub fn summarize_grant_drafts_for_metric(items: &[grant_draft::GrantDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_limit_metrics_from_draft(items: &[grant_draft::GrantDraft]) -> Vec<LimitMetric> {
    let mut out: Vec<LimitMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_limit_metric(item: &LimitMetric) -> String {
    grant_draft::normalize_grant_metric_code(&item.code)
}
