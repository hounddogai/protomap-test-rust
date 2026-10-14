use super::*;

use std::collections::BTreeMap;

use crate::credential::estimate as credential_estimate;

pub fn summarize_scheduled_credential_segments_for_adjustment(
    items: &[credential_estimate::ScheduledCredentialSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_token_adjustments_from_segment(
    items: &[credential_estimate::ScheduledCredentialSegment],
) -> Vec<TokenAdjustment> {
    let mut out: Vec<TokenAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_token_adjustment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_token_adjustment(item: &TokenAdjustment) -> String {
    credential_estimate::normalize_manual_credential_quota_code(&item.code)
}
