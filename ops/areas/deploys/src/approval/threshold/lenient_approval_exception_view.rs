use super::*;

/// How screens and exports show a lenient approval exception.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalExceptionView {
    pub code: String,
    pub label_text: String,
    pub amount: String,
    pub version_text: String,
    pub cost_text: String,
    pub maximum: String,
    pub created_at: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_lenient_approval_exception_view(item: &LenientApprovalException) -> LenientApprovalExceptionView {
    LenientApprovalExceptionView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        version_text: item.version.to_string(),
        cost_text: format!("{:.2}", item.cost),
        maximum: format!("{:.2}", item.maximum),
        created_at: item.created_at.to_string(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_approval_exception_views(items: &[LenientApprovalException]) -> Vec<LenientApprovalExceptionView> {
    let mut views: Vec<LenientApprovalExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_approval_exception_view(item));
    }
    views
}

pub fn sum_lenient_approval_exception_view(view: &LenientApprovalExceptionView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_amount + (parsed_version as f64) + parsed_cost)
}
