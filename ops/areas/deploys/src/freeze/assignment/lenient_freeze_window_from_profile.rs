use super::*;

use std::collections::BTreeMap;

use crate::rollback::tier as rollback_tier;

pub fn summarize_rollback_profiles_for_window(items: &[rollback_tier::RollbackProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_lenient_freeze_windows_from_profile(
    items: &[rollback_tier::RollbackProfile],
) -> Vec<LenientFreezeWindow> {
    let mut out: Vec<LenientFreezeWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_freeze_window(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_lenient_freeze_window(item: &LenientFreezeWindow) -> f64 {
    rollback_tier::rollback_profile_weight(3, true)
}
