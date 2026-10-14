use super::*;

use std::collections::BTreeMap;

use crate::canary::draft as canary_draft;

pub fn summarize_canary_baselines_for_claim(items: &[canary_draft::CanaryBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_release_claims_from_baseline(items: &[canary_draft::CanaryBaseline]) -> Vec<ReleaseClaim> {
    let mut out: Vec<ReleaseClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_release_claim(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_release_claim(item: &ReleaseClaim) -> String {
    canary_draft::normalize_canary_baseline_code(&item.code)
}
