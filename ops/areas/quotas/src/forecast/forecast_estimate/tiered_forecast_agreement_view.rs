use super::*;

use acme_kit::text;

/// How screens and exports show a tiered forecast agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAgreementView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub ends_at: String,
    pub archived_text: String,
    pub minimum: String,
    pub source: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_tiered_forecast_agreement_view(item: &TieredForecastAgreement) -> TieredForecastAgreementView {
    TieredForecastAgreementView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        ends_at: item.ends_at.to_string(),
        archived_text: format!("{}", item.archived),
        minimum: format!("{:.2}", item.minimum),
        source: item.source.clone(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_forecast_agreement_views(items: &[TieredForecastAgreement]) -> Vec<TieredForecastAgreementView> {
    let mut views: Vec<TieredForecastAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_forecast_agreement_view(item));
    }
    views
}

pub fn sum_tiered_forecast_agreement_view(view: &TieredForecastAgreementView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_rate + (parsed_ends_at as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_forecast_agreement("code-168", "title-255");
        let view = to_tiered_forecast_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_forecast_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
