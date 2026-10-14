use super::*;

use std::collections::BTreeMap;

use crate::federation::federation_request as federation_federation_request;

pub fn summarize_scheduled_federation_requests_for_policy(
    items: &[federation_federation_request::ScheduledFederationRequest],
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

pub fn derive_strict_identity_policies_from_request(
    items: &[federation_federation_request::ScheduledFederationRequest],
) -> Vec<StrictIdentityPolicy> {
    let mut out: Vec<StrictIdentityPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_identity_policy(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_strict_identity_policy(item: &StrictIdentityPolicy) -> String {
    federation_federation_request::normalize_regional_federation_agreement_code(&item.code)
}
