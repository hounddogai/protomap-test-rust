use super::*;

use acme_kit::text;

/// How screens and exports show a global federation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationForecastView {
    pub code: String,
    pub name: String,
    pub updated_at_text: String,
    pub reviewer_text: String,
    pub locale: String,
    pub attempts: String,
    pub cipher: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_global_federation_forecast_view(item: &GlobalFederationForecast) -> GlobalFederationForecastView {
    GlobalFederationForecastView {
        code: item.code.clone(),
        name: item.name.clone(),
        updated_at_text: item.updated_at.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 64),
        locale: text::truncate(&item.locale, 64),
        attempts: item.attempts.to_string(),
        cipher: item.cipher.clone(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_federation_forecast_views(items: &[GlobalFederationForecast]) -> Vec<GlobalFederationForecastView> {
    let mut views: Vec<GlobalFederationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_federation_forecast_view(item));
    }
    views
}

pub fn sum_global_federation_forecast_view(view: &GlobalFederationForecastView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_federation_forecast("code-556", "name-584");
        let view = to_global_federation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_federation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
