use super::*;

use std::collections::BTreeMap;

use crate::isolation::segment as isolation_segment;

pub fn summarize_isolation_adjustments_for_exception(
    items: &[isolation_segment::IsolationAdjustment],
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

pub fn derive_default_isolation_exceptions_from_adjustment(
    items: &[isolation_segment::IsolationAdjustment],
) -> Vec<DefaultIsolationException> {
    let mut out: Vec<DefaultIsolationException> = Vec::new();
    for item in items.iter() {
        let derived = new_default_isolation_exception(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_default_isolation_exception_extra(item: &DefaultIsolationException) -> String {
    isolation_segment::classify_global_isolation_score(item.effective_value())
}
