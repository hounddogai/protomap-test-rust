use super::*;

use acme_kit::text;

/// How screens and exports show a weighted trust domain baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainBaselineView {
    pub code_text: String,
    pub name: String,
    pub cost: String,
    pub quota_units: String,
    pub namespace: String,
    pub approved: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_weighted_trust_domain_baseline_view(item: &WeightedTrustDomainBaseline) -> WeightedTrustDomainBaselineView {
    WeightedTrustDomainBaselineView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        cost: format!("{:.2}", item.cost),
        quota_units: item.quota_units.to_string(),
        namespace: item.namespace.clone(),
        approved: format!("{}", item.approved),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_trust_domain_baseline_views(
    items: &[WeightedTrustDomainBaseline],
) -> Vec<WeightedTrustDomainBaselineView> {
    let mut views: Vec<WeightedTrustDomainBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_trust_domain_baseline_view(item));
    }
    views
}

pub fn sum_weighted_trust_domain_baseline_view(view: &WeightedTrustDomainBaselineView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_cost + (parsed_quota_units as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_trust_domain_baseline("code-12", "name-138");
        let view = to_weighted_trust_domain_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_trust_domain_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
