use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneView {
    pub code: String,
    pub title: String,
    pub ratio_text: String,
    pub score: String,
    pub category: String,
    pub contact_email_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_ttl_policy_milestone_view(item: &TtlPolicyMilestone) -> TtlPolicyMilestoneView {
    TtlPolicyMilestoneView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        ratio_text: format!("{:.2}", item.ratio),
        score: format!("{:.2}", item.score),
        category: text::truncate(&item.category, 120),
        contact_email_text: text::truncate(&item.contact_email, 64),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_milestone_views(items: &[TtlPolicyMilestone]) -> Vec<TtlPolicyMilestoneView> {
    let mut views: Vec<TtlPolicyMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_milestone_view(item));
    }
    views
}

pub fn sum_ttl_policy_milestone_view(view: &TtlPolicyMilestoneView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ratio + parsed_score + (parsed_expires_at as f64))
}
