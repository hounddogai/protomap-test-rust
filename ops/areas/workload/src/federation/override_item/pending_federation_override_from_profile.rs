use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_milestone as binding_binding_milestone;

pub fn summarize_binding_profiles_for_override(
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

pub fn derive_pending_federation_overrides_from_profile(
    items: &[binding_binding_milestone::BindingProfile],
) -> Vec<PendingFederationOverride> {
    let mut out: Vec<PendingFederationOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_federation_override(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_pending_federation_override(item: &PendingFederationOverride) -> String {
    binding_binding_milestone::classify_fallback_binding_milestone(item.projected_value())
}
