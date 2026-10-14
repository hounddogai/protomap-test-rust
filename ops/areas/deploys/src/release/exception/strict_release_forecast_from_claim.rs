use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_agreement as approval_approval_agreement;

pub fn summarize_approval_claims_for_forecast(
    items: &[approval_approval_agreement::ApprovalClaim],
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

pub fn derive_strict_release_forecasts_from_claim(
    items: &[approval_approval_agreement::ApprovalClaim],
) -> Vec<StrictReleaseForecast> {
    let mut out: Vec<StrictReleaseForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_release_forecast(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_strict_release_forecast(item: &StrictReleaseForecast) -> f64 {
    approval_approval_agreement::approval_claim_weight(4, true)
}
