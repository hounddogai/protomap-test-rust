use super::*;

use std::collections::BTreeMap;

use crate::identity::targets as identity_targets;

pub fn summarize_pending_identity_summaries_for_proposal(
    items: &[identity_targets::PendingIdentitySummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_legacy_identity_proposals_from_summary(
    items: &[identity_targets::PendingIdentitySummary],
) -> Vec<LegacyIdentityProposal> {
    let mut out: Vec<LegacyIdentityProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_identity_proposal(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_legacy_identity_proposal(item: &LegacyIdentityProposal) -> f64 {
    identity_targets::pending_identity_summary_weight(2, true)
}
