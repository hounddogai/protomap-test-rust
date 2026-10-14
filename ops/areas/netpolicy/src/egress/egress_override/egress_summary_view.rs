use super::*;

/// How screens and exports show a egress summary.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummaryView {
    pub code: String,
    pub title_text: String,
    pub pinned: String,
    pub expires_at: String,
    pub burst_text: String,
    pub description_text: String,
    pub locale: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_egress_summary_view(item: &EgressSummary) -> EgressSummaryView {
    EgressSummaryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        pinned: format!("{}", item.pinned),
        expires_at: item.expires_at.to_string(),
        burst_text: item.burst.to_string(),
        description_text: item.description.clone(),
        locale: item.locale.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_summary_views(items: &[EgressSummary]) -> Vec<EgressSummaryView> {
    let mut views: Vec<EgressSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_summary_view(item));
    }
    views
}

pub fn sum_egress_summary_view(view: &EgressSummaryView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_burst as f64))
}
