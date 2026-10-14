use super::*;

use acme_kit::text;

/// How screens and exports show a manual rule review.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleReviewView {
    pub code_text: String,
    pub label: String,
    pub version: String,
    pub revision_text: String,
    pub country: String,
    pub created_at: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_manual_rule_review_view(item: &ManualRuleReview) -> ManualRuleReviewView {
    ManualRuleReviewView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        version: item.version.to_string(),
        revision_text: item.revision.to_string(),
        country: text::truncate(&item.country, 40),
        created_at: item.created_at.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_rule_review_views(items: &[ManualRuleReview]) -> Vec<ManualRuleReviewView> {
    let mut views: Vec<ManualRuleReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_rule_review_view(item));
    }
    views
}

pub fn sum_manual_rule_review_view(view: &ManualRuleReviewView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_revision as f64) + (parsed_created_at as f64))
}
