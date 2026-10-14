use super::*;

/// How screens and exports show a fallback isolation record.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecordView {
    pub code: String,
    pub title_text: String,
    pub scope_text: String,
    pub ttl_seconds: String,
    pub minimum: String,
    pub budget_text: String,
    pub principal_text: String,
    pub score: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_fallback_isolation_record_view(item: &FallbackIsolationRecord) -> FallbackIsolationRecordView {
    FallbackIsolationRecordView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        scope_text: item.scope.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        minimum: format!("{:.2}", item.minimum),
        budget_text: format!("{:.2}", item.budget),
        principal_text: item.principal.clone(),
        score: format!("{:.2}", item.score),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_isolation_record_views(items: &[FallbackIsolationRecord]) -> Vec<FallbackIsolationRecordView> {
    let mut views: Vec<FallbackIsolationRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_isolation_record_view(item));
    }
    views
}

pub fn sum_fallback_isolation_record_view(view: &FallbackIsolationRecordView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_minimum + parsed_budget)
}
