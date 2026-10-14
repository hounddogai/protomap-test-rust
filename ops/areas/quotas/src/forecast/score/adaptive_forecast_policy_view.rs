use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive forecast policy.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastPolicyView {
    pub code_text: String,
    pub title: String,
    pub updated_at_text: String,
    pub rate: String,
    pub country: String,
    pub approved: String,
    pub quantity: String,
    pub budget: String,
    pub cluster: String,
    pub source: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_adaptive_forecast_policy_view(item: &AdaptiveForecastPolicy) -> AdaptiveForecastPolicyView {
    AdaptiveForecastPolicyView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        updated_at_text: item.updated_at.to_string(),
        rate: format!("{:.2}", item.rate),
        country: item.country.clone(),
        approved: format!("{}", item.approved),
        quantity: item.quantity.to_string(),
        budget: format!("{:.2}", item.budget),
        cluster: item.cluster.clone(),
        source: text::truncate(&item.source, 120),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_forecast_policy_views(items: &[AdaptiveForecastPolicy]) -> Vec<AdaptiveForecastPolicyView> {
    let mut views: Vec<AdaptiveForecastPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_forecast_policy_view(item));
    }
    views
}

pub fn sum_adaptive_forecast_policy_view(view: &AdaptiveForecastPolicyView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_rate + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_forecast_policy("code-21", "title-221");
        let view = to_adaptive_forecast_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_forecast_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
