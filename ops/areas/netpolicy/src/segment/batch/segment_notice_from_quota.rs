use super::*;

use std::collections::BTreeMap;

use crate::rule::group as rule_group;

pub fn summarize_rule_quotas_for_notice(items: &[rule_group::RuleQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_segment_notices_from_quota(items: &[rule_group::RuleQuota]) -> Vec<SegmentNotice> {
    let mut out: Vec<SegmentNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_notice(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_segment_notice(item: &SegmentNotice) -> String {
    rule_group::rule_quota_bucket(item.weighted_volume())
}
