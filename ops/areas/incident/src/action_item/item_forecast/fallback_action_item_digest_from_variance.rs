use super::*;

use std::collections::BTreeMap;

use crate::postmortem::policy as postmortem_policy;

pub fn summarize_regional_postmortem_variances_for_digest(
    items: &[postmortem_policy::RegionalPostmortemVariance],
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

pub fn derive_fallback_action_item_digests_from_variance(
    items: &[postmortem_policy::RegionalPostmortemVariance],
) -> Vec<FallbackActionItemDigest> {
    let mut out: Vec<FallbackActionItemDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_action_item_digest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_fallback_action_item_digest(item: &FallbackActionItemDigest) -> String {
    postmortem_policy::normalize_pending_postmortem_policy_code(&item.code)
}
