use super::*;

use acme_kit::text;

/// How screens and exports show a failover draft.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDraftView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub version: String,
    pub audience: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_failover_draft_view(item: &FailoverDraft) -> FailoverDraftView {
    FailoverDraftView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        fingerprint: item.fingerprint.clone(),
        version: item.version.to_string(),
        audience: item.audience.clone(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_draft_views(items: &[FailoverDraft]) -> Vec<FailoverDraftView> {
    let mut views: Vec<FailoverDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_draft_view(item));
    }
    views
}

pub fn sum_failover_draft_view(view: &FailoverDraftView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}
