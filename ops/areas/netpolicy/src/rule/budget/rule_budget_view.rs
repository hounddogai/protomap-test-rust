use super::*;

use acme_kit::text;

/// How screens and exports show a rule budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBudgetView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub score: String,
    pub channel_text: String,
    pub external_ref: String,
    pub approved: String,
    pub reviewer: String,
    pub floor_value: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_rule_budget_view(item: &RuleBudget) -> RuleBudgetView {
    RuleBudgetView {
        code: item.code.clone(),
        label: item.label.clone(),
        currency: item.currency.clone(),
        score: format!("{:.2}", item.score),
        channel_text: text::truncate(&item.channel, 40),
        external_ref: item.external_ref.clone(),
        approved: format!("{}", item.approved),
        reviewer: item.reviewer.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_budget_views(items: &[RuleBudget]) -> Vec<RuleBudgetView> {
    let mut views: Vec<RuleBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_budget_view(item));
    }
    views
}

pub fn sum_rule_budget_view(view: &RuleBudgetView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_score + parsed_floor_value + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_budget("code-745", "label-367");
        let view = to_rule_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
