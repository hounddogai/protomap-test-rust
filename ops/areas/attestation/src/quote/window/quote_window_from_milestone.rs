use super::*;

use std::collections::BTreeMap;

use crate::verifier::baseline as verifier_baseline;

pub fn summarize_verifier_milestones_for_window(
    items: &[verifier_baseline::VerifierMilestone],
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

pub fn derive_quote_windows_from_milestone(items: &[verifier_baseline::VerifierMilestone]) -> Vec<QuoteWindow> {
    let mut out: Vec<QuoteWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_window(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_quote_window(item: &QuoteWindow) -> String {
    verifier_baseline::capped_verifier_baseline_bucket(item.projected_volume())
}
