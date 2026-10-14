use super::*;

use std::collections::BTreeMap;

use crate::freeze::metric as freeze_metric;

pub fn summarize_freeze_templates_for_proposal(items: &[freeze_metric::FreezeTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_release_proposals_from_template(items: &[freeze_metric::FreezeTemplate]) -> Vec<ReleaseProposal> {
    let mut out: Vec<ReleaseProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_release_proposal(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_release_proposal(item: &ReleaseProposal) -> f64 {
    freeze_metric::freeze_template_weight(7, true)
}
