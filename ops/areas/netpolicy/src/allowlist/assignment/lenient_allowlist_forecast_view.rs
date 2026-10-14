use super::*;

use acme_kit::text;

/// How screens and exports show a lenient allowlist forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecastView {
    pub code: String,
    pub title_text: String,
    pub locale: String,
    pub version_text: String,
    pub scope: String,
    pub country: String,
    pub host_text: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_lenient_allowlist_forecast_view(item: &LenientAllowlistForecast) -> LenientAllowlistForecastView {
    LenientAllowlistForecastView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        locale: text::truncate(&item.locale, 40),
        version_text: item.version.to_string(),
        scope: item.scope.clone(),
        country: item.country.clone(),
        host_text: item.host.clone(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_allowlist_forecast_views(items: &[LenientAllowlistForecast]) -> Vec<LenientAllowlistForecastView> {
    let mut views: Vec<LenientAllowlistForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_allowlist_forecast_view(item));
    }
    views
}

pub fn sum_lenient_allowlist_forecast_view(view: &LenientAllowlistForecastView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_allowlist_forecast("code-466", "title-914");
        let view = to_lenient_allowlist_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_allowlist_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
