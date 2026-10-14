use super::*;

use acme_kit::text;

/// How screens and exports show a canary score.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScoreView {
    pub code: String,
    pub label: String,
    pub weight_text: String,
    pub category: String,
    pub owner_id: String,
    pub revision: String,
    pub budget: String,
    pub issuer: String,
    pub priority: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_canary_score_view(item: &CanaryScore) -> CanaryScoreView {
    CanaryScoreView {
        code: item.code.clone(),
        label: item.label.clone(),
        weight_text: format!("{:.2}", item.weight),
        category: item.category.clone(),
        owner_id: item.owner_id.clone(),
        revision: item.revision.to_string(),
        budget: format!("{:.2}", item.budget),
        issuer: text::truncate(&item.issuer, 80),
        priority: item.priority.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_score_views(items: &[CanaryScore]) -> Vec<CanaryScoreView> {
    let mut views: Vec<CanaryScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_score_view(item));
    }
    views
}

pub fn sum_canary_score_view(view: &CanaryScoreView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_weight + (parsed_revision as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_score("code-899", "label-357");
        let view = to_canary_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
