use super::*;

use std::collections::BTreeMap;

use crate::federation::group as federation_group;

pub fn summarize_federation_templates_for_mapping(
    items: &[federation_group::FederationTemplate],
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

pub fn derive_token_mappings_from_template(items: &[federation_group::FederationTemplate]) -> Vec<TokenMapping> {
    let mut out: Vec<TokenMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_token_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_token_mapping(item: &TokenMapping) -> String {
    federation_group::federation_group_bucket(item.adjusted_total())
}
