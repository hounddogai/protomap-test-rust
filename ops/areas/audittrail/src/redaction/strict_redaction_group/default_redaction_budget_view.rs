use super::*;

use acme_kit::text;

/// How screens and exports show a default redaction budget.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionBudgetView {
    pub code: String,
    pub title: String,
    pub updated_at_text: String,
    pub port: String,
    pub description: String,
    pub cluster: String,
    pub threshold: String,
    pub quantity_text: String,
    pub weight_text: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_default_redaction_budget_view(item: &DefaultRedactionBudget) -> DefaultRedactionBudgetView {
    DefaultRedactionBudgetView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        updated_at_text: item.updated_at.to_string(),
        port: item.port.to_string(),
        description: text::truncate(&item.description, 120),
        cluster: item.cluster.clone(),
        threshold: format!("{:.2}", item.threshold),
        quantity_text: item.quantity.to_string(),
        weight_text: format!("{:.2}", item.weight),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_redaction_budget_views(items: &[DefaultRedactionBudget]) -> Vec<DefaultRedactionBudgetView> {
    let mut views: Vec<DefaultRedactionBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_redaction_budget_view(item));
    }
    views
}

pub fn sum_default_redaction_budget_view(view: &DefaultRedactionBudgetView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_port as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_redaction_budget("code-159", "title-812");
        let view = to_default_redaction_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_redaction_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
