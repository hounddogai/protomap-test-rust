use super::*;

use acme_kit::text;

/// How screens and exports show a redaction budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudgetView {
    pub code: String,
    pub name: String,
    pub threshold: String,
    pub priority: String,
    pub revision: String,
    pub segment: String,
    pub rate_text: String,
    pub external_ref: String,
    pub burst_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_redaction_budget_view(item: &RedactionBudget) -> RedactionBudgetView {
    RedactionBudgetView {
        code: item.code.clone(),
        name: item.name.clone(),
        threshold: format!("{:.2}", item.threshold),
        priority: item.priority.to_string(),
        revision: item.revision.to_string(),
        segment: item.segment.clone(),
        rate_text: format!("{:.2}", item.rate),
        external_ref: item.external_ref.clone(),
        burst_text: item.burst.to_string(),
        notes: text::truncate(&item.notes, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_budget_views(items: &[RedactionBudget]) -> Vec<RedactionBudgetView> {
    let mut views: Vec<RedactionBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_budget_view(item));
    }
    views
}

pub fn sum_redaction_budget_view(view: &RedactionBudgetView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_threshold + (parsed_priority as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_budget("code-945", "name-81");
        let view = to_redaction_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
