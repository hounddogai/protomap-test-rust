use super::*;

use acme_kit::text;

/// How screens and exports show a lenient ttl policy budget.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyBudgetView {
    pub code: String,
    pub title: String,
    pub category: String,
    pub margin: String,
    pub score: String,
    pub description_text: String,
    pub status_label: String,
}

pub fn to_lenient_ttl_policy_budget_view(item: &LenientTtlPolicyBudget) -> LenientTtlPolicyBudgetView {
    LenientTtlPolicyBudgetView {
        code: item.code.clone(),
        title: item.title.clone(),
        category: text::truncate(&item.category, 120),
        margin: format!("{:.2}", item.margin),
        score: format!("{:.2}", item.score),
        description_text: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_ttl_policy_budget_views(items: &[LenientTtlPolicyBudget]) -> Vec<LenientTtlPolicyBudgetView> {
    let mut views: Vec<LenientTtlPolicyBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_ttl_policy_budget_view(item));
    }
    views
}

pub fn sum_lenient_ttl_policy_budget_view(view: &LenientTtlPolicyBudgetView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_margin + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_ttl_policy_budget("code-217", "title-847");
        let view = to_lenient_ttl_policy_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_ttl_policy_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
