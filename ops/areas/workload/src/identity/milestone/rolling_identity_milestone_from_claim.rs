use super::*;

use std::collections::BTreeMap;

use crate::token::adjustment as token_adjustment;

pub fn summarize_capped_token_claims_for_milestone(
    items: &[token_adjustment::CappedTokenClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_rolling_identity_milestones_from_claim(
    items: &[token_adjustment::CappedTokenClaim],
) -> Vec<RollingIdentityMilestone> {
    let mut out: Vec<RollingIdentityMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_identity_milestone(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_rolling_identity_milestone(item: &RollingIdentityMilestone) -> String {
    token_adjustment::lenient_token_reservation_bucket(item.weighted_volume())
}
