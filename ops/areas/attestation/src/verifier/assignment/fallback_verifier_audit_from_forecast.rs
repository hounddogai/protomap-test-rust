use super::*;

use std::collections::BTreeMap;

use crate::policy::policy_preference as policy_policy_preference;

pub fn summarize_strict_policy_forecasts_for_audit(
    items: &[policy_policy_preference::StrictPolicyForecast],
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

pub fn derive_fallback_verifier_audits_from_forecast(
    items: &[policy_policy_preference::StrictPolicyForecast],
) -> Vec<FallbackVerifierAudit> {
    let mut out: Vec<FallbackVerifierAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_verifier_audit(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_fallback_verifier_audit(item: &FallbackVerifierAudit) -> String {
    policy_policy_preference::classify_lenient_policy_preference(item.weighted_exposure())
}
