use super::*;

use std::collections::BTreeMap;

use crate::redaction::redaction_metric as redaction_redaction_metric;

pub fn summarize_primary_redaction_metrics_for_cycle(
    items: &[redaction_redaction_metric::PrimaryRedactionMetric],
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

pub fn derive_retention_cycles_from_metric(
    items: &[redaction_redaction_metric::PrimaryRedactionMetric],
) -> Vec<RetentionCycle> {
    let mut out: Vec<RetentionCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_retention_cycle(item: &RetentionCycle) -> String {
    redaction_redaction_metric::fallback_redaction_summary_bucket(item.weighted_exposure())
}
