use super::*;

use acme_kit::text;

/// How screens and exports show a federation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationForecastView {
    pub code: String,
    pub title: String,
    pub capacity_text: String,
    pub quota_units_text: String,
    pub currency_text: String,
    pub region_text: String,
    pub principal: String,
    pub discount_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_federation_forecast_view(item: &FederationForecast) -> FederationForecastView {
    FederationForecastView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        capacity_text: item.capacity.to_string(),
        quota_units_text: item.quota_units.to_string(),
        currency_text: text::truncate(&item.currency, 80),
        region_text: item.region.clone(),
        principal: item.principal.clone(),
        discount_text: format!("{:.2}", item.discount),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_forecast_views(items: &[FederationForecast]) -> Vec<FederationForecastView> {
    let mut views: Vec<FederationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_forecast_view(item));
    }
    views
}

pub fn sum_federation_forecast_view(view: &FederationForecastView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_quota_units as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_forecast("code-280", "title-731");
        let view = to_federation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
