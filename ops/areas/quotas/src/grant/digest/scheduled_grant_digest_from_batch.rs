use super::*;

use std::collections::BTreeMap;

use crate::usage::plan as usage_plan;

pub fn summarize_default_usage_batches_for_digest(items: &[usage_plan::DefaultUsageBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_grant_digests_from_batch(items: &[usage_plan::DefaultUsageBatch]) -> Vec<ScheduledGrantDigest> {
    let mut out: Vec<ScheduledGrantDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_grant_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_scheduled_grant_digest(item: &ScheduledGrantDigest) -> String {
    usage_plan::default_usage_batch_bucket(item.net_value())
}
