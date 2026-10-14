use super::*;

use std::collections::BTreeMap;

use crate::egress::cycle as egress_cycle;

pub fn summarize_egress_notices_for_window(items: &[egress_cycle::EgressNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_segment_windows_from_notice(items: &[egress_cycle::EgressNotice]) -> Vec<SegmentWindow> {
    let mut out: Vec<SegmentWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_segment_window(item: &SegmentWindow) -> String {
    egress_cycle::legacy_egress_rollup_bucket(item.adjusted_exposure())
}
