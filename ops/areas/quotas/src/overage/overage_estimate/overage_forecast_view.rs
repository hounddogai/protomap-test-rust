use super::*;

use acme_kit::text;

/// How screens and exports show a overage forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageForecastView {
    pub code: String,
    pub label: String,
    pub name_text: String,
    pub host: String,
    pub duration_minutes: String,
    pub version: String,
    pub cipher: String,
    pub expires_at_text: String,
    pub audience: String,
    pub quota_units: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_overage_forecast_view(item: &OverageForecast) -> OverageForecastView {
    OverageForecastView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        name_text: text::truncate(&item.name, 40),
        host: item.host.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        version: item.version.to_string(),
        cipher: text::truncate(&item.cipher, 80),
        expires_at_text: item.expires_at.to_string(),
        audience: item.audience.clone(),
        quota_units: item.quota_units.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_forecast_views(items: &[OverageForecast]) -> Vec<OverageForecastView> {
    let mut views: Vec<OverageForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_forecast_view(item));
    }
    views
}

pub fn sum_overage_forecast_view(view: &OverageForecastView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_version as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_forecast("code-805", "label-939");
        let view = to_overage_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
