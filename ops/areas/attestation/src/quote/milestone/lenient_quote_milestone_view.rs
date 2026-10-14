use super::*;

/// How screens and exports show a lenient quote milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteMilestoneView {
    pub code: String,
    pub name: String,
    pub window_days: String,
    pub priority: String,
    pub amount_text: String,
    pub issuer_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_lenient_quote_milestone_view(item: &LenientQuoteMilestone) -> LenientQuoteMilestoneView {
    LenientQuoteMilestoneView {
        code: item.code.clone(),
        name: item.name.clone(),
        window_days: item.window_days.to_string(),
        priority: item.priority.to_string(),
        amount_text: format!("{:.2}", item.amount),
        issuer_text: item.issuer.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_quote_milestone_views(items: &[LenientQuoteMilestone]) -> Vec<LenientQuoteMilestoneView> {
    let mut views: Vec<LenientQuoteMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_quote_milestone_view(item));
    }
    views
}

pub fn sum_lenient_quote_milestone_view(view: &LenientQuoteMilestoneView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_priority as f64) + parsed_amount)
}
