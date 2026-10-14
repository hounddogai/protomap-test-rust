use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::template as trust_domain_template;

pub fn summarize_capped_trust_domain_scores_for_segment(
    items: &[trust_domain_template::CappedTrustDomainScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_binding_segments_from_score(
    items: &[trust_domain_template::CappedTrustDomainScore],
) -> Vec<BindingSegment> {
    let mut out: Vec<BindingSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_segment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_binding_segment(item: &BindingSegment) -> String {
    trust_domain_template::capped_trust_domain_score_bucket(item.net_total())
}
