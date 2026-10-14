use super::*;

use std::collections::BTreeMap;

use crate::grant::draft as grant_draft;

pub fn summarize_grant_metrics_for_agreement(items: &[grant_draft::GrantMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_usage_agreements_from_metric(items: &[grant_draft::GrantMetric]) -> Vec<UsageAgreement> {
    let mut out: Vec<UsageAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_usage_agreement(item: &UsageAgreement) -> String {
    grant_draft::normalize_grant_draft_code(&item.code)
}
