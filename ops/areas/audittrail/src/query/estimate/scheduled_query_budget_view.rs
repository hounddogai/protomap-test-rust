use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled query budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryBudgetView {
    pub code_text: String,
    pub label_text: String,
    pub version_text: String,
    pub maximum_text: String,
    pub margin_text: String,
    pub expires_at_text: String,
    pub created_at: String,
    pub tenant_id: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_scheduled_query_budget_view(item: &ScheduledQueryBudget) -> ScheduledQueryBudgetView {
    ScheduledQueryBudgetView {
        code_text: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        version_text: item.version.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        margin_text: format!("{:.2}", item.margin),
        expires_at_text: item.expires_at.to_string(),
        created_at: item.created_at.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_query_budget_views(items: &[ScheduledQueryBudget]) -> Vec<ScheduledQueryBudgetView> {
    let mut views: Vec<ScheduledQueryBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_query_budget_view(item));
    }
    views
}

pub fn sum_scheduled_query_budget_view(view: &ScheduledQueryBudgetView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_version as f64 + parsed_maximum + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_query_budget("code-943", "label-614");
        let view = to_scheduled_query_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_query_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
