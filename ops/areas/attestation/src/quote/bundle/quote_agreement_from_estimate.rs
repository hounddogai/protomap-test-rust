use super::*;

use std::collections::BTreeMap;

use crate::policy::summary as policy_summary;

pub fn summarize_policy_estimates_for_agreement(items: &[policy_summary::PolicyEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_quote_agreements_from_estimate(items: &[policy_summary::PolicyEstimate]) -> Vec<QuoteAgreement> {
    let mut out: Vec<QuoteAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_agreement(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_quote_agreement(item: &QuoteAgreement) -> String {
    policy_summary::classify_policy_window(item.net_load())
}
