use super::*;

use std::collections::BTreeMap;

use crate::release::template as release_template;

pub fn summarize_release_templates_for_proposal(items: &[release_template::ReleaseTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_global_freeze_proposals_from_template(
    items: &[release_template::ReleaseTemplate],
) -> Vec<GlobalFreezeProposal> {
    let mut out: Vec<GlobalFreezeProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_global_freeze_proposal(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_global_freeze_proposal(item: &GlobalFreezeProposal) -> String {
    release_template::normalize_release_template_code(&item.code)
}
