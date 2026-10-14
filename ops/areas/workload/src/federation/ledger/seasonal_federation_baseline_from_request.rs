use super::*;

use std::collections::BTreeMap;

use crate::token::draft as token_draft;

pub fn summarize_token_requests_for_baseline(items: &[token_draft::TokenRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_seasonal_federation_baselines_from_request(
    items: &[token_draft::TokenRequest],
) -> Vec<SeasonalFederationBaseline> {
    let mut out: Vec<SeasonalFederationBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_federation_baseline(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_seasonal_federation_baseline(item: &SeasonalFederationBaseline) -> f64 {
    token_draft::token_window_weight(6, true)
}
