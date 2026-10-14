use super::*;

use std::collections::BTreeMap;

use crate::alert::preference as alert_preference;

pub fn summarize_alert_thresholds_for_draft(items: &[alert_preference::AlertThreshold]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_probe_drafts_from_threshold(items: &[alert_preference::AlertThreshold]) -> Vec<ProbeDraft> {
    let mut out: Vec<ProbeDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_probe_draft(item: &ProbeDraft) -> String {
    alert_preference::classify_alert_preference(item.net_total())
}
