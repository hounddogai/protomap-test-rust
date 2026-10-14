use super::*;

use acme_kit::text;

/// How screens and exports show a grant forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastView {
    pub code: String,
    pub name: String,
    pub score: String,
    pub budget_text: String,
    pub quota_units_text: String,
    pub limit_text: String,
    pub external_ref: String,
    pub checksum_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_grant_forecast_view(item: &GrantForecast) -> GrantForecastView {
    GrantForecastView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        score: format!("{:.2}", item.score),
        budget_text: format!("{:.2}", item.budget),
        quota_units_text: item.quota_units.to_string(),
        limit_text: item.limit.to_string(),
        external_ref: item.external_ref.clone(),
        checksum_text: text::truncate(&item.checksum, 120),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_forecast_views(items: &[GrantForecast]) -> Vec<GrantForecastView> {
    let mut views: Vec<GrantForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_forecast_view(item));
    }
    views
}

pub fn sum_grant_forecast_view(view: &GrantForecastView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_score + parsed_budget + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_forecast("code-214", "name-614");
        let view = to_grant_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
