use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_claim as escalation_escalation_claim;

pub fn summarize_rolling_escalation_claims_for_quota(
    items: &[escalation_escalation_claim::RollingEscalationClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_page_quotas_from_claim(items: &[escalation_escalation_claim::RollingEscalationClaim]) -> Vec<PageQuota> {
    let mut out: Vec<PageQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_page_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_page_quota(item: &PageQuota) -> String {
    escalation_escalation_claim::lenient_escalation_batch_bucket(item.measured_value())
}
