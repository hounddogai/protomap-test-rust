use super::*;

use std::collections::BTreeMap;

use crate::verifier::mapping as verifier_mapping;

pub fn summarize_verifier_mappings_for_baseline(items: &[verifier_mapping::VerifierMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_verifier_baselines_from_mapping(items: &[verifier_mapping::VerifierMapping]) -> Vec<VerifierBaseline> {
    let mut out: Vec<VerifierBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_baseline(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_verifier_baseline(item: &VerifierBaseline) -> String {
    verifier_mapping::verifier_mapping_bucket(item.net_volume())
}
