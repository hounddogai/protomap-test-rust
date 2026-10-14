use super::*;

use std::collections::BTreeMap;

use crate::grant::review as grant_review;

pub fn summarize_grant_bundles_for_bundle(items: &[grant_review::GrantBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_seasonal_overage_bundles_from_bundle(items: &[grant_review::GrantBundle]) -> Vec<SeasonalOverageBundle> {
    let mut out: Vec<SeasonalOverageBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_overage_bundle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_seasonal_overage_bundle(item: &SeasonalOverageBundle) -> String {
    grant_review::normalize_fallback_grant_agreement_code(&item.code)
}
