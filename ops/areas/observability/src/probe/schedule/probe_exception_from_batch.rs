use super::*;

use std::collections::BTreeMap;

use crate::probe::request as probe_request;

pub fn summarize_probe_batches_for_exception(items: &[probe_request::ProbeBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_probe_exceptions_from_batch(items: &[probe_request::ProbeBatch]) -> Vec<ProbeException> {
    let mut out: Vec<ProbeException> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_exception(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_probe_exception(item: &ProbeException) -> String {
    probe_request::normalize_probe_override_code(&item.code)
}
