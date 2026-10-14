use super::*;

use acme_kit::text;

/// How screens and exports show a tenant review.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantReviewView {
    pub code_text: String,
    pub name: String,
    pub archived_text: String,
    pub locale: String,
    pub source_text: String,
    pub phone: String,
    pub category: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_tenant_review_view(item: &TenantReview) -> TenantReviewView {
    TenantReviewView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        archived_text: format!("{}", item.archived),
        locale: item.locale.clone(),
        source_text: item.source.clone(),
        phone: item.phone.clone(),
        category: text::truncate(&item.category, 80),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_review_views(items: &[TenantReview]) -> Vec<TenantReviewView> {
    let mut views: Vec<TenantReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_review_view(item));
    }
    views
}

pub fn sum_tenant_review_view(view: &TenantReviewView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
