use super::*;

use acme_kit::text;

/// How screens and exports show a identity forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityForecastView {
    pub code_text: String,
    pub label: String,
    pub description: String,
    pub cost_text: String,
    pub name_text: String,
    pub status_label: String,
}

pub fn to_identity_forecast_view(item: &IdentityForecast) -> IdentityForecastView {
    IdentityForecastView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        description: item.description.clone(),
        cost_text: format!("{:.2}", item.cost),
        name_text: text::truncate(&item.name, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_forecast_views(items: &[IdentityForecast]) -> Vec<IdentityForecastView> {
    let mut views: Vec<IdentityForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_forecast_view(item));
    }
    views
}

pub fn sum_identity_forecast_view(view: &IdentityForecastView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_forecast("code-505", "label-932");
        let view = to_identity_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
