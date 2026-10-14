use super::*;

/// How screens and exports show a postmortem audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAuditView {
    pub code: String,
    pub label_text: String,
    pub description_text: String,
    pub score: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_postmortem_audit_view(item: &PostmortemAudit) -> PostmortemAuditView {
    PostmortemAuditView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        description_text: item.description.clone(),
        score: format!("{:.2}", item.score),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_audit_views(items: &[PostmortemAudit]) -> Vec<PostmortemAuditView> {
    let mut views: Vec<PostmortemAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_audit_view(item));
    }
    views
}

pub fn sum_postmortem_audit_view(view: &PostmortemAuditView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
