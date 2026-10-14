use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_exception as isolation_isolation_exception;

pub fn summarize_isolation_profiles_for_segment(
    items: &[isolation_isolation_exception::IsolationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_pending_suspension_segments_from_profile(
    items: &[isolation_isolation_exception::IsolationProfile],
) -> Vec<PendingSuspensionSegment> {
    let mut out: Vec<PendingSuspensionSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_suspension_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_suspension_segment(item: &PendingSuspensionSegment) -> String {
    isolation_isolation_exception::classify_default_isolation_exception(item.adjusted_value())
}
