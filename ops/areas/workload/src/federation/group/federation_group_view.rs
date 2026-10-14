use super::*;

/// How screens and exports show a federation group.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationGroupView {
    pub code: String,
    pub name_text: String,
    pub amount: String,
    pub budget_text: String,
    pub weight_text: String,
    pub scope: String,
    pub approved: String,
    pub timezone_text: String,
    pub ceiling: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_federation_group_view(item: &FederationGroup) -> FederationGroupView {
    FederationGroupView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        budget_text: format!("{:.2}", item.budget),
        weight_text: format!("{:.2}", item.weight),
        scope: item.scope.clone(),
        approved: format!("{}", item.approved),
        timezone_text: item.timezone.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_group_views(items: &[FederationGroup]) -> Vec<FederationGroupView> {
    let mut views: Vec<FederationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_group_view(item));
    }
    views
}

pub fn sum_federation_group_view(view: &FederationGroupView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_amount + parsed_budget + parsed_weight)
}
