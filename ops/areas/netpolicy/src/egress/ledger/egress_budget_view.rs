use super::*;

use acme_kit::text;

/// How screens and exports show a egress budget.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudgetView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub ends_at: String,
    pub ratio_text: String,
    pub visible: String,
    pub channel: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_egress_budget_view(item: &EgressBudget) -> EgressBudgetView {
    EgressBudgetView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        score: format!("{:.2}", item.score),
        ends_at: item.ends_at.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        visible: format!("{}", item.visible),
        channel: text::truncate(&item.channel, 64),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_budget_views(items: &[EgressBudget]) -> Vec<EgressBudgetView> {
    let mut views: Vec<EgressBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_budget_view(item));
    }
    views
}

pub fn sum_egress_budget_view(view: &EgressBudgetView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_score + (parsed_ends_at as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_budget("code-707", "title-589");
        let view = to_egress_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
