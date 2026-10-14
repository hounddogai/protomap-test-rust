use super::*;

use std::collections::BTreeMap;

use crate::token::baseline as token_baseline;

pub fn summarize_seasonal_token_metrics_for_slot(
    items: &[token_baseline::SeasonalTokenMetric],
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

pub fn derive_seasonal_credential_slots_from_metric(
    items: &[token_baseline::SeasonalTokenMetric],
) -> Vec<SeasonalCredentialSlot> {
    let mut out: Vec<SeasonalCredentialSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_credential_slot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_seasonal_credential_slot(item: &SeasonalCredentialSlot) -> String {
    token_baseline::manual_token_proposal_bucket(item.effective_total())
}
