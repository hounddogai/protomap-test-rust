use super::*;

/// How screens and exports show a default limit tier.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitTierView {
    pub code_text: String,
    pub title: String,
    pub limit: String,
    pub ttl_seconds: String,
    pub retries_text: String,
    pub key_id_text: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_default_limit_tier_view(item: &DefaultLimitTier) -> DefaultLimitTierView {
    DefaultLimitTierView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        limit: item.limit.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        retries_text: item.retries.to_string(),
        key_id_text: item.key_id.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_limit_tier_views(items: &[DefaultLimitTier]) -> Vec<DefaultLimitTierView> {
    let mut views: Vec<DefaultLimitTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_limit_tier_view(item));
    }
    views
}

pub fn sum_default_limit_tier_view(view: &DefaultLimitTierView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_ttl_seconds as f64) + (parsed_retries as f64))
}
