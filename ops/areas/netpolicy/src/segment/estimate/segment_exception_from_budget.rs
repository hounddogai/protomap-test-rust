use super::*;

use std::collections::BTreeMap;

use crate::egress::budget as egress_budget;

pub fn summarize_lenient_egress_budgets_for_exception(
    items: &[egress_budget::LenientEgressBudget],
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

pub fn derive_segment_exceptions_from_budget(items: &[egress_budget::LenientEgressBudget]) -> Vec<SegmentException> {
    let mut out: Vec<SegmentException> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_exception(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_segment_exception(item: &SegmentException) -> String {
    egress_budget::seasonal_egress_schedule_bucket(item.effective_volume())
}
