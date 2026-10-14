use super::*;

use std::collections::BTreeMap;

use crate::nonce::record_item as nonce_record_item;

pub fn summarize_rolling_nonce_baselines_for_summary(
    items: &[nonce_record_item::RollingNonceBaseline],
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

pub fn derive_manual_evidence_summaries_from_baseline(
    items: &[nonce_record_item::RollingNonceBaseline],
) -> Vec<ManualEvidenceSummary> {
    let mut out: Vec<ManualEvidenceSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_evidence_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_manual_evidence_summary(item: &ManualEvidenceSummary) -> String {
    nonce_record_item::normalize_rolling_nonce_baseline_code(&item.code)
}
