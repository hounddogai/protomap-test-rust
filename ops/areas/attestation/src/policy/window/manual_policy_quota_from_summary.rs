use super::*;

use std::collections::BTreeMap;

use crate::quote::summary as quote_summary;

pub fn summarize_quote_summaries_for_quota(items: &[quote_summary::QuoteSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_manual_policy_quotas_from_summary(items: &[quote_summary::QuoteSummary]) -> Vec<ManualPolicyQuota> {
    let mut out: Vec<ManualPolicyQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_policy_quota(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_manual_policy_quota(item: &ManualPolicyQuota) -> f64 {
    quote_summary::global_quote_checkpoint_weight(2, true)
}
