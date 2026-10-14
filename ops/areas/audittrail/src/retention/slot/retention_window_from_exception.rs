use super::*;

use std::collections::BTreeMap;

use crate::sink::quota as sink_quota;

pub fn summarize_sink_exceptions_for_window(items: &[sink_quota::SinkException]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_retention_windows_from_exception(items: &[sink_quota::SinkException]) -> Vec<RetentionWindow> {
    let mut out: Vec<RetentionWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_retention_window(item: &RetentionWindow) -> String {
    sink_quota::normalize_sink_quota_code(&item.code)
}
