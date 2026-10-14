use super::*;

use std::collections::BTreeMap;

use crate::quote::targets as quote_targets;

pub fn summarize_quote_audits_for_policy(items: &[quote_targets::QuoteAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_scheduled_measurement_policies_from_audit(
    items: &[quote_targets::QuoteAudit],
) -> Vec<ScheduledMeasurementPolicy> {
    let mut out: Vec<ScheduledMeasurementPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_measurement_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_scheduled_measurement_policy(item: &ScheduledMeasurementPolicy) -> String {
    quote_targets::quote_audit_bucket(item.effective_volume())
}
