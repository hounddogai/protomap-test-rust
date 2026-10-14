use super::*;

use std::collections::BTreeMap;

use crate::retention::baseline as retention_baseline;

pub fn summarize_retention_baselines_for_summary(
    items: &[retention_baseline::RetentionBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_fallback_redaction_summaries_from_baseline(
    items: &[retention_baseline::RetentionBaseline],
) -> Vec<FallbackRedactionSummary> {
    let mut out: Vec<FallbackRedactionSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_redaction_summary(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_fallback_redaction_summary(item: &FallbackRedactionSummary) -> String {
    retention_baseline::classify_retention_claim(item.adjusted_value())
}
