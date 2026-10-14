use super::*;

use std::collections::BTreeMap;

use crate::freeze::assignment as freeze_assignment;

pub fn summarize_freeze_assignments_for_segment(
    items: &[freeze_assignment::FreezeAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_release_segments_from_assignment(items: &[freeze_assignment::FreezeAssignment]) -> Vec<ReleaseSegment> {
    let mut out: Vec<ReleaseSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_release_segment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_release_segment(item: &ReleaseSegment) -> String {
    freeze_assignment::normalize_lenient_freeze_window_code(&item.code)
}
