use super::*;

use std::collections::BTreeMap;

use crate::identity::window as identity_window;

pub fn summarize_identity_windows_for_plan(items: &[identity_window::IdentityWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_strict_token_plans_from_window(items: &[identity_window::IdentityWindow]) -> Vec<StrictTokenPlan> {
    let mut out: Vec<StrictTokenPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_token_plan(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_strict_token_plan(item: &StrictTokenPlan) -> String {
    identity_window::scheduled_identity_window_bucket(item.projected_exposure())
}
