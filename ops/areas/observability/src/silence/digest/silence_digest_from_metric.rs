use super::*;

use std::collections::BTreeMap;

use crate::probe::plan as probe_plan;
use crate::silence::bundle as silence_bundle;

pub fn summarize_silence_metrics_for_digest(items: &[silence_bundle::SilenceMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_silence_digests_from_metric(items: &[silence_bundle::SilenceMetric]) -> Vec<SilenceDigest> {
    let mut out: Vec<SilenceDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_silence_digest(item: &SilenceDigest) -> String {
    silence_bundle::manual_silence_bundle_bucket(item.weighted_total())
}

pub fn summarize_probe_metrics_for_digest(items: &[probe_plan::ProbeMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_silence_digests_from_metric_extra(items: &[probe_plan::ProbeMetric]) -> Vec<SilenceDigest> {
    let mut out: Vec<SilenceDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_silence_digest(item: &SilenceDigest) -> String {
    probe_plan::probe_plan_bucket(item.weighted_total())
}
