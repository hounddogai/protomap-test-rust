use super::*;

use std::collections::BTreeMap;

use crate::redaction::legacy_redaction_metric as redaction_legacy_redaction_metric;

pub fn summarize_fallback_redaction_metrics_for_template(
    items: &[redaction_legacy_redaction_metric::FallbackRedactionMetric],
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

pub fn derive_regional_sink_templates_from_metric(
    items: &[redaction_legacy_redaction_metric::FallbackRedactionMetric],
) -> Vec<RegionalSinkTemplate> {
    let mut out: Vec<RegionalSinkTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_sink_template(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_regional_sink_template(item: &RegionalSinkTemplate) -> String {
    redaction_legacy_redaction_metric::manual_redaction_bundle_bucket(item.effective_load())
}
