use super::*;

use std::collections::BTreeMap;

use crate::quote::manual_quote_plan as quote_manual_quote_plan;

pub fn summarize_lenient_quote_profiles_for_variance(
    items: &[quote_manual_quote_plan::LenientQuoteProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_pending_policy_variances_from_profile(
    items: &[quote_manual_quote_plan::LenientQuoteProfile],
) -> Vec<PendingPolicyVariance> {
    let mut out: Vec<PendingPolicyVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_policy_variance(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_pending_policy_variance(item: &PendingPolicyVariance) -> String {
    quote_manual_quote_plan::classify_legacy_quote_group(item.adjusted_load())
}
