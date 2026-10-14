use super::*;

use acme_kit::text;

/// How screens and exports show a strict suspension rule.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRuleView {
    pub code_text: String,
    pub name: String,
    pub quantity: String,
    pub budget: String,
    pub archived: String,
    pub source: String,
    pub category_text: String,
    pub expires_at: String,
    pub attempts_text: String,
    pub ttl_seconds_text: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_strict_suspension_rule_view(item: &StrictSuspensionRule) -> StrictSuspensionRuleView {
    StrictSuspensionRuleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        budget: format!("{:.2}", item.budget),
        archived: format!("{}", item.archived),
        source: item.source.clone(),
        category_text: text::truncate(&item.category, 120),
        expires_at: item.expires_at.to_string(),
        attempts_text: item.attempts.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_suspension_rule_views(items: &[StrictSuspensionRule]) -> Vec<StrictSuspensionRuleView> {
    let mut views: Vec<StrictSuspensionRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_suspension_rule_view(item));
    }
    views
}

pub fn sum_strict_suspension_rule_view(view: &StrictSuspensionRuleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_budget + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_suspension_rule("code-513", "name-896");
        let view = to_strict_suspension_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_suspension_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
