use super::*;

use acme_kit::text;

/// How screens and exports show a primary overage forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecastView {
    pub code: String,
    pub label: String,
    pub host_text: String,
    pub updated_at: String,
    pub issuer: String,
    pub reviewer_text: String,
    pub currency: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_primary_overage_forecast_view(item: &PrimaryOverageForecast) -> PrimaryOverageForecastView {
    PrimaryOverageForecastView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        host_text: item.host.clone(),
        updated_at: item.updated_at.to_string(),
        issuer: text::truncate(&item.issuer, 120),
        reviewer_text: text::truncate(&item.reviewer, 80),
        currency: item.currency.clone(),
        external_ref_text: text::truncate(&item.external_ref, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_overage_forecast_views(items: &[PrimaryOverageForecast]) -> Vec<PrimaryOverageForecastView> {
    let mut views: Vec<PrimaryOverageForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_overage_forecast_view(item));
    }
    views
}

pub fn sum_primary_overage_forecast_view(view: &PrimaryOverageForecastView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_overage_forecast("code-591", "label-981");
        let view = to_primary_overage_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_overage_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
