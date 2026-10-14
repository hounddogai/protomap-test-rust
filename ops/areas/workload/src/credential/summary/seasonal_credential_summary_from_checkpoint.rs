use super::*;

use std::collections::BTreeMap;

use crate::identity::window as identity_window;

pub fn summarize_lenient_identity_checkpoints_for_summary(
    items: &[identity_window::LenientIdentityCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_seasonal_credential_summaries_from_checkpoint(
    items: &[identity_window::LenientIdentityCheckpoint],
) -> Vec<SeasonalCredentialSummary> {
    let mut out: Vec<SeasonalCredentialSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_credential_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_seasonal_credential_summary_extra(item: &SeasonalCredentialSummary) -> String {
    identity_window::scheduled_identity_window_bucket(item.net_total())
}
