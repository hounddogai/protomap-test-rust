use super::*;

use acme_kit::text;

/// How screens and exports show a delegation estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimateView {
    pub code_text: String,
    pub label: String,
    pub budget_text: String,
    pub score_text: String,
    pub archived_text: String,
    pub quantity_text: String,
    pub locale: String,
    pub region: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_delegation_estimate_view(item: &DelegationEstimate) -> DelegationEstimateView {
    DelegationEstimateView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        budget_text: format!("{:.2}", item.budget),
        score_text: format!("{:.2}", item.score),
        archived_text: format!("{}", item.archived),
        quantity_text: item.quantity.to_string(),
        locale: item.locale.clone(),
        region: text::truncate(&item.region, 64),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_estimate_views(items: &[DelegationEstimate]) -> Vec<DelegationEstimateView> {
    let mut views: Vec<DelegationEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_estimate_view(item));
    }
    views
}

pub fn sum_delegation_estimate_view(view: &DelegationEstimateView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_budget + parsed_score + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_estimate("code-248", "label-884");
        let view = to_delegation_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
