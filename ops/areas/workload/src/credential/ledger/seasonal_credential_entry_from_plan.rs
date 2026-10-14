use super::*;

use std::collections::BTreeMap;

use crate::credential::credential_bundle as credential_credential_bundle;

pub fn summarize_credential_plans_for_entry(
    items: &[credential_credential_bundle::CredentialPlan],
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

pub fn derive_seasonal_credential_entries_from_plan(
    items: &[credential_credential_bundle::CredentialPlan],
) -> Vec<SeasonalCredentialEntry> {
    let mut out: Vec<SeasonalCredentialEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_credential_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_seasonal_credential_entry(item: &SeasonalCredentialEntry) -> String {
    credential_credential_bundle::default_credential_mapping_bucket(item.measured_value())
}
