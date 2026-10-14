use super::*;

use std::collections::BTreeMap;

use crate::identity::window as identity_window;

pub fn summarize_identity_milestones_for_agreement(
    items: &[identity_window::IdentityMilestone],
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

pub fn derive_lenient_federation_agreements_from_milestone(
    items: &[identity_window::IdentityMilestone],
) -> Vec<LenientFederationAgreement> {
    let mut out: Vec<LenientFederationAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_federation_agreement(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_lenient_federation_agreement(item: &LenientFederationAgreement) -> String {
    identity_window::batched_identity_target_bucket(item.measured_value())
}
