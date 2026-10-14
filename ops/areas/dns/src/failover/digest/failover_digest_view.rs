use super::*;

use acme_kit::text;

/// How screens and exports show a failover digest.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDigestView {
    pub code_text: String,
    pub title: String,
    pub threshold_text: String,
    pub burst: String,
    pub host_text: String,
    pub updated_at: String,
    pub margin: String,
    pub name: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_failover_digest_view(item: &FailoverDigest) -> FailoverDigestView {
    FailoverDigestView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        threshold_text: format!("{:.2}", item.threshold),
        burst: item.burst.to_string(),
        host_text: item.host.clone(),
        updated_at: item.updated_at.to_string(),
        margin: format!("{:.2}", item.margin),
        name: item.name.clone(),
        namespace: text::truncate(&item.namespace, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_digest_views(items: &[FailoverDigest]) -> Vec<FailoverDigestView> {
    let mut views: Vec<FailoverDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_digest_view(item));
    }
    views
}

pub fn sum_failover_digest_view(view: &FailoverDigestView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_threshold + (parsed_burst as f64) + (parsed_updated_at as f64))
}
