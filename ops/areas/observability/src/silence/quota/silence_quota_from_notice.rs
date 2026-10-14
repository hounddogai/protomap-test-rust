use super::*;

use std::collections::BTreeMap;

use crate::probe::agreement as probe_agreement;

pub fn summarize_probe_notices_for_quota(items: &[probe_agreement::ProbeNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_silence_quotas_from_notice(items: &[probe_agreement::ProbeNotice]) -> Vec<SilenceQuota> {
    let mut out: Vec<SilenceQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_quota(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_silence_quota(item: &SilenceQuota) -> String {
    probe_agreement::normalize_pending_probe_request_code(&item.code)
}
