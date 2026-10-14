use super::*;

use std::collections::BTreeMap;

use crate::postmortem::claim as postmortem_claim;

pub fn summarize_global_postmortem_claims_for_profile(
    items: &[postmortem_claim::GlobalPostmortemClaim],
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

pub fn derive_strict_escalation_profiles_from_claim(
    items: &[postmortem_claim::GlobalPostmortemClaim],
) -> Vec<StrictEscalationProfile> {
    let mut out: Vec<StrictEscalationProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_escalation_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_strict_escalation_profile(item: &StrictEscalationProfile) -> f64 {
    postmortem_claim::fallback_postmortem_review_weight(7, true)
}
