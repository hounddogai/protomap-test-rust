use super::*;

use std::collections::BTreeMap;

use crate::suspension::agreement as suspension_agreement;

pub fn summarize_adaptive_suspension_baselines_for_bundle(
    items: &[suspension_agreement::AdaptiveSuspensionBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_suspension_bundles_from_baseline(
    items: &[suspension_agreement::AdaptiveSuspensionBaseline],
) -> Vec<SuspensionBundle> {
    let mut out: Vec<SuspensionBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_bundle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_suspension_bundle(item: &SuspensionBundle) -> String {
    suspension_agreement::normalize_suspension_audit_code(&item.code)
}
