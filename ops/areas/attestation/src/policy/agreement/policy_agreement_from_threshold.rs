use super::*;

use std::collections::BTreeMap;

use crate::quote::quote_plan as quote_quote_plan;

pub fn summarize_quote_thresholds_for_agreement(items: &[quote_quote_plan::QuoteThreshold]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_policy_agreements_from_threshold(items: &[quote_quote_plan::QuoteThreshold]) -> Vec<PolicyAgreement> {
    let mut out: Vec<PolicyAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_agreement(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_policy_agreement(item: &PolicyAgreement) -> String {
    quote_quote_plan::classify_manual_quote_plan(item.net_value())
}
