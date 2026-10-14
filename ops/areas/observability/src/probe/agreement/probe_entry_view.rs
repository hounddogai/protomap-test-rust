use super::*;

use acme_kit::text;

/// How screens and exports show a probe entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEntryView {
    pub code: String,
    pub title: String,
    pub sequence: String,
    pub discount: String,
    pub key_id: String,
    pub minimum: String,
    pub threshold: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_probe_entry_view(item: &ProbeEntry) -> ProbeEntryView {
    ProbeEntryView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        sequence: item.sequence.to_string(),
        discount: format!("{:.2}", item.discount),
        key_id: item.key_id.clone(),
        minimum: format!("{:.2}", item.minimum),
        threshold: format!("{:.2}", item.threshold),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_entry_views(items: &[ProbeEntry]) -> Vec<ProbeEntryView> {
    let mut views: Vec<ProbeEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_entry_view(item));
    }
    views
}

pub fn sum_probe_entry_view(view: &ProbeEntryView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_discount + parsed_minimum)
}
