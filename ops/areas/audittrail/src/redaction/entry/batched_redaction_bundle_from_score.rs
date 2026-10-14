use super::*;

use std::collections::BTreeMap;

use crate::retention::reservation as retention_reservation;

pub fn summarize_retention_scores_for_bundle(items: &[retention_reservation::RetentionScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_batched_redaction_bundles_from_score(
    items: &[retention_reservation::RetentionScore],
) -> Vec<BatchedRedactionBundle> {
    let mut out: Vec<BatchedRedactionBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_redaction_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_batched_redaction_bundle(item: &BatchedRedactionBundle) -> f64 {
    retention_reservation::retention_reservation_weight(6, true)
}
