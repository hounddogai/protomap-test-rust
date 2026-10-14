use super::*;

use std::collections::BTreeMap;

use crate::query::segment as query_segment;

pub fn summarize_query_quotas_for_milestone(items: &[query_segment::QueryQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_redaction_milestones_from_quota(items: &[query_segment::QueryQuota]) -> Vec<RedactionMilestone> {
    let mut out: Vec<RedactionMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_milestone(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_redaction_milestone(item: &RedactionMilestone) -> String {
    query_segment::normalize_query_segment_code(&item.code)
}
