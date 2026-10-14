use super::*;

use acme_kit::text;

/// How screens and exports show a strict policy item.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyItemView {
    pub code_text: String,
    pub name: String,
    pub source: String,
    pub summary: String,
    pub weight: String,
    pub limit_text: String,
    pub scope: String,
    pub budget: String,
    pub burst: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_strict_policy_item_view(item: &StrictPolicyItem) -> StrictPolicyItemView {
    StrictPolicyItemView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        source: text::truncate(&item.source, 40),
        summary: text::truncate(&item.summary, 80),
        weight: format!("{:.2}", item.weight),
        limit_text: item.limit.to_string(),
        scope: text::truncate(&item.scope, 64),
        budget: format!("{:.2}", item.budget),
        burst: item.burst.to_string(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_policy_item_views(items: &[StrictPolicyItem]) -> Vec<StrictPolicyItemView> {
    let mut views: Vec<StrictPolicyItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_policy_item_view(item));
    }
    views
}

pub fn sum_strict_policy_item_view(view: &StrictPolicyItemView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_weight + (parsed_limit as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_policy_item("code-816", "name-66");
        let view = to_strict_policy_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_policy_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
