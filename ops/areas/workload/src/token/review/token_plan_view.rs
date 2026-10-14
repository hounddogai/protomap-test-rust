use super::*;

use acme_kit::text;

/// How screens and exports show a token plan.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenPlanView {
    pub code: String,
    pub name_text: String,
    pub notes_text: String,
    pub cost: String,
    pub quota_units: String,
    pub currency: String,
    pub port_text: String,
    pub owner_id: String,
    pub created_at: String,
    pub issuer: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_token_plan_view(item: &TokenPlan) -> TokenPlanView {
    TokenPlanView {
        code: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 120),
        notes_text: text::truncate(&item.notes, 40),
        cost: format!("{:.2}", item.cost),
        quota_units: item.quota_units.to_string(),
        currency: item.currency.clone(),
        port_text: item.port.to_string(),
        owner_id: item.owner_id.clone(),
        created_at: item.created_at.to_string(),
        issuer: text::truncate(&item.issuer, 120),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_plan_views(items: &[TokenPlan]) -> Vec<TokenPlanView> {
    let mut views: Vec<TokenPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_plan_view(item));
    }
    views
}

pub fn sum_token_plan_view(view: &TokenPlanView) -> Result<f64, String> {
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
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_cost + (parsed_quota_units as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_plan("code-101", "name-584");
        let view = to_token_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
