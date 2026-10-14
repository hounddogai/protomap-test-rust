use super::*;

use acme_kit::text;

/// How screens and exports show a nonce exception.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceExceptionView {
    pub code: String,
    pub title: String,
    pub ttl_seconds: String,
    pub capacity: String,
    pub created_at_text: String,
    pub grace_days_text: String,
    pub burst_text: String,
    pub floor_value: String,
    pub reviewer: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_nonce_exception_view(item: &NonceException) -> NonceExceptionView {
    NonceExceptionView {
        code: item.code.clone(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        capacity: item.capacity.to_string(),
        created_at_text: item.created_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        burst_text: item.burst.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        reviewer: item.reviewer.clone(),
        label: text::truncate(&item.label, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_exception_views(items: &[NonceException]) -> Vec<NonceExceptionView> {
    let mut views: Vec<NonceExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_exception_view(item));
    }
    views
}

pub fn sum_nonce_exception_view(view: &NonceExceptionView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_capacity as f64) + (parsed_created_at as f64))
}
