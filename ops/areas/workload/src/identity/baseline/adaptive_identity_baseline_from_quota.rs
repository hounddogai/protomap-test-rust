use super::*;

use std::collections::BTreeMap;

use crate::credential::estimate as credential_estimate;

pub fn summarize_manual_credential_quotas_for_baseline(
    items: &[credential_estimate::ManualCredentialQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_adaptive_identity_baselines_from_quota(
    items: &[credential_estimate::ManualCredentialQuota],
) -> Vec<AdaptiveIdentityBaseline> {
    let mut out: Vec<AdaptiveIdentityBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_identity_baseline(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_adaptive_identity_baseline(item: &AdaptiveIdentityBaseline) -> String {
    credential_estimate::classify_scheduled_credential_segment(item.weighted_load())
}
