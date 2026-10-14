use super::*;

use std::collections::BTreeMap;

use crate::probe::draft as probe_draft;

pub fn summarize_probe_drafts_for_group(items: &[probe_draft::ProbeDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_silence_groups_from_draft(items: &[probe_draft::ProbeDraft]) -> Vec<SilenceGroup> {
    let mut out: Vec<SilenceGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_group(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_silence_group_extra(item: &SilenceGroup) -> String {
    probe_draft::fallback_probe_tier_bucket(item.measured_value())
}
