use super::*;

use std::collections::BTreeMap;

use crate::plan::draft as plan_draft;

pub fn summarize_plan_windows_for_review(items: &[plan_draft::PlanWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_isolation_reviews_from_window(items: &[plan_draft::PlanWindow]) -> Vec<IsolationReview> {
    let mut out: Vec<IsolationReview> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_isolation_review(item: &IsolationReview) -> f64 {
    plan_draft::plan_window_weight(2, true)
}
