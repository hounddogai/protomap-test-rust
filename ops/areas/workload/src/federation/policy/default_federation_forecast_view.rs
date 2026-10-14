use super::*;

use acme_kit::text;

/// How screens and exports show a default federation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationForecastView {
    pub code_text: String,
    pub label: String,
    pub scope: String,
    pub external_ref_text: String,
    pub quantity: String,
    pub visible: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_default_federation_forecast_view(item: &DefaultFederationForecast) -> DefaultFederationForecastView {
    DefaultFederationForecastView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        scope: item.scope.clone(),
        external_ref_text: text::truncate(&item.external_ref, 40),
        quantity: item.quantity.to_string(),
        visible: format!("{}", item.visible),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_federation_forecast_views(items: &[DefaultFederationForecast]) -> Vec<DefaultFederationForecastView> {
    let mut views: Vec<DefaultFederationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_federation_forecast_view(item));
    }
    views
}

pub fn sum_default_federation_forecast_view(view: &DefaultFederationForecastView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_federation_forecast("code-765", "label-941");
        let view = to_default_federation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_federation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
