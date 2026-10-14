use super::*;

use std::collections::BTreeMap;

use crate::quote::draft as quote_draft;

pub fn summarize_legacy_quote_agreements_for_summary(
    items: &[quote_draft::LegacyQuoteAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_policy_summaries_from_agreement(
    items: &[quote_draft::LegacyQuoteAgreement],
) -> Vec<ScheduledPolicySummary> {
    let mut out: Vec<ScheduledPolicySummary> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_policy_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_scheduled_policy_summary(item: &ScheduledPolicySummary) -> String {
    quote_draft::quote_segment_bucket(item.weighted_volume())
}
