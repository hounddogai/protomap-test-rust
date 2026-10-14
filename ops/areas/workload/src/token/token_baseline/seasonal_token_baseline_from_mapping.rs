use super::*;

use std::collections::BTreeMap;

use crate::credential::credential_bundle as credential_credential_bundle;

pub fn summarize_default_credential_mappings_for_baseline(
    items: &[credential_credential_bundle::DefaultCredentialMapping],
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

pub fn derive_seasonal_token_baselines_from_mapping(
    items: &[credential_credential_bundle::DefaultCredentialMapping],
) -> Vec<SeasonalTokenBaseline> {
    let mut out: Vec<SeasonalTokenBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_token_baseline(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_seasonal_token_baseline(item: &SeasonalTokenBaseline) -> String {
    credential_credential_bundle::classify_credential_plan(item.effective_volume())
}
