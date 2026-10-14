use super::*;

/// How screens and exports show a suspension exception.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionExceptionView {
    pub code_text: String,
    pub name: String,
    pub quota_units_text: String,
    pub visible: String,
    pub label: String,
    pub amount: String,
    pub limit_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_suspension_exception_view(item: &SuspensionException) -> SuspensionExceptionView {
    SuspensionExceptionView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        quota_units_text: item.quota_units.to_string(),
        visible: format!("{}", item.visible),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        limit_text: item.limit.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_exception_views(items: &[SuspensionException]) -> Vec<SuspensionExceptionView> {
    let mut views: Vec<SuspensionExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_exception_view(item));
    }
    views
}

pub fn sum_suspension_exception_view(view: &SuspensionExceptionView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_amount + (parsed_limit as f64))
}
