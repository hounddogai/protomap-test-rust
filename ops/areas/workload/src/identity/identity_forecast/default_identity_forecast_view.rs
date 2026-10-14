use super::*;

use acme_kit::text;

/// How screens and exports show a default identity forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityForecastView {
    pub code: String,
    pub label_text: String,
    pub category: String,
    pub duration_minutes: String,
    pub visible: String,
    pub burst: String,
    pub phone: String,
    pub amount: String,
    pub ttl_seconds_text: String,
    pub notes_text: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_default_identity_forecast_view(item: &DefaultIdentityForecast) -> DefaultIdentityForecastView {
    DefaultIdentityForecastView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        category: item.category.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        visible: format!("{}", item.visible),
        burst: item.burst.to_string(),
        phone: text::truncate(&item.phone, 80),
        amount: format!("{:.2}", item.amount),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        notes_text: item.notes.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_identity_forecast_views(items: &[DefaultIdentityForecast]) -> Vec<DefaultIdentityForecastView> {
    let mut views: Vec<DefaultIdentityForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_identity_forecast_view(item));
    }
    views
}

pub fn sum_default_identity_forecast_view(view: &DefaultIdentityForecastView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_burst as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_identity_forecast("code-980", "label-872");
        let view = to_default_identity_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_identity_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
