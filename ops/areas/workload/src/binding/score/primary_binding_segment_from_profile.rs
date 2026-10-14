use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_milestone as binding_binding_milestone;

pub fn summarize_binding_profiles_for_segment(
    items: &[binding_binding_milestone::BindingProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_primary_binding_segments_from_profile(
    items: &[binding_binding_milestone::BindingProfile],
) -> Vec<PrimaryBindingSegment> {
    let mut out: Vec<PrimaryBindingSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_binding_segment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_primary_binding_segment(item: &PrimaryBindingSegment) -> String {
    binding_binding_milestone::binding_profile_bucket(item.projected_volume())
}
