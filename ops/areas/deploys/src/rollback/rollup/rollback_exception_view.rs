use super::*;

use acme_kit::text;

/// How screens and exports show a rollback exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackExceptionView {
    pub code: String,
    pub name_text: String,
    pub cipher: String,
    pub cost_text: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_rollback_exception_view(item: &RollbackException) -> RollbackExceptionView {
    RollbackExceptionView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        cipher: text::truncate(&item.cipher, 120),
        cost_text: format!("{:.2}", item.cost),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_exception_views(items: &[RollbackException]) -> Vec<RollbackExceptionView> {
    let mut views: Vec<RollbackExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_exception_view(item));
    }
    views
}

pub fn sum_rollback_exception_view(view: &RollbackExceptionView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_cost + (parsed_retries as f64))
}
