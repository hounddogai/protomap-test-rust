use super::*;

use std::collections::BTreeMap;

use crate::query::legacy_query_milestone as query_legacy_query_milestone;

pub fn summarize_adaptive_query_adjustments_for_metric(
    items: &[query_legacy_query_milestone::AdaptiveQueryAdjustment],
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

pub fn derive_redaction_metrics_from_adjustment(
    items: &[query_legacy_query_milestone::AdaptiveQueryAdjustment],
) -> Vec<RedactionMetric> {
    let mut out: Vec<RedactionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_metric(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_redaction_metric(item: &RedactionMetric) -> String {
    query_legacy_query_milestone::normalize_default_query_adjustment_code(&item.code)
}
