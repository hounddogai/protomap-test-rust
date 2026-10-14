use super::*;

use std::collections::BTreeMap;

use crate::rule::tier as rule_tier;

pub fn summarize_rule_milestones_for_record(items: &[rule_tier::RuleMilestone]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_segment_records_from_milestone(items: &[rule_tier::RuleMilestone]) -> Vec<SegmentRecord> {
    let mut out: Vec<SegmentRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_segment_record(item: &SegmentRecord) -> String {
    rule_tier::normalize_rule_bundle_code(&item.code)
}
