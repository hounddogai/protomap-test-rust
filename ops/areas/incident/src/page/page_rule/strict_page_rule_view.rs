use super::*;

use acme_kit::text;

/// How screens and exports show a strict page rule.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRuleView {
    pub code_text: String,
    pub title: String,
    pub scope_text: String,
    pub unit: String,
    pub budget_text: String,
    pub region_text: String,
    pub notes: String,
    pub grace_days: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_strict_page_rule_view(item: &StrictPageRule) -> StrictPageRuleView {
    StrictPageRuleView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        scope_text: item.scope.clone(),
        unit: text::truncate(&item.unit, 40),
        budget_text: format!("{:.2}", item.budget),
        region_text: text::truncate(&item.region, 120),
        notes: item.notes.clone(),
        grace_days: item.grace_days.to_string(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_page_rule_views(items: &[StrictPageRule]) -> Vec<StrictPageRuleView> {
    let mut views: Vec<StrictPageRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_page_rule_view(item));
    }
    views
}

pub fn sum_strict_page_rule_view(view: &StrictPageRuleView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_budget + (parsed_grace_days as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_page_rule("code-958", "title-635");
        let view = to_strict_page_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_page_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
