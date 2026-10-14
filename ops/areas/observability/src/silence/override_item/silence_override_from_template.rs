use super::*;

use std::collections::BTreeMap;

use crate::probe::template as probe_template;

pub fn summarize_probe_templates_for_override(items: &[probe_template::ProbeTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_silence_overrides_from_template(items: &[probe_template::ProbeTemplate]) -> Vec<SilenceOverride> {
    let mut out: Vec<SilenceOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_override(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_silence_override(item: &SilenceOverride) -> String {
    probe_template::probe_baseline_bucket(item.net_volume())
}
