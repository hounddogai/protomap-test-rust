use super::*;

use acme_kit::text;

/// How screens and exports show a probe baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBaselineView {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub expires_at: String,
    pub reviewer: String,
    pub cluster_text: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_probe_baseline_view(item: &ProbeBaseline) -> ProbeBaselineView {
    ProbeBaselineView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        owner_id: item.owner_id.clone(),
        expires_at: item.expires_at.to_string(),
        reviewer: item.reviewer.clone(),
        cluster_text: text::truncate(&item.cluster, 80),
        description: text::truncate(&item.description, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_baseline_views(items: &[ProbeBaseline]) -> Vec<ProbeBaselineView> {
    let mut views: Vec<ProbeBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_baseline_view(item));
    }
    views
}

pub fn sum_probe_baseline_view(view: &ProbeBaselineView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}
