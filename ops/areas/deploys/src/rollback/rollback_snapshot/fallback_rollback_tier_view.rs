use super::*;

use acme_kit::text;

/// How screens and exports show a fallback rollback tier.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierView {
    pub code_text: String,
    pub title: String,
    pub retries_text: String,
    pub ratio_text: String,
    pub limit: String,
    pub quota_units: String,
    pub description: String,
    pub weight: String,
    pub duration_minutes: String,
    pub segment_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_fallback_rollback_tier_view(item: &FallbackRollbackTier) -> FallbackRollbackTierView {
    FallbackRollbackTierView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        retries_text: item.retries.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        limit: item.limit.to_string(),
        quota_units: item.quota_units.to_string(),
        description: text::truncate(&item.description, 40),
        weight: format!("{:.2}", item.weight),
        duration_minutes: item.duration_minutes.to_string(),
        segment_text: text::truncate(&item.segment, 80),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_rollback_tier_views(items: &[FallbackRollbackTier]) -> Vec<FallbackRollbackTierView> {
    let mut views: Vec<FallbackRollbackTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_rollback_tier_view(item));
    }
    views
}

pub fn sum_fallback_rollback_tier_view(view: &FallbackRollbackTierView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_retries as f64 + parsed_ratio + (parsed_limit as f64))
}
