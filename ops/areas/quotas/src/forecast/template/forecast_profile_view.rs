use super::*;

use acme_kit::text;

/// How screens and exports show a forecast profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfileView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub ceiling: String,
    pub category: String,
    pub country_text: String,
    pub fingerprint: String,
    pub attempts: String,
    pub notes: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_forecast_profile_view(item: &ForecastProfile) -> ForecastProfileView {
    ForecastProfileView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 80),
        ends_at: item.ends_at.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        category: text::truncate(&item.category, 120),
        country_text: item.country.clone(),
        fingerprint: item.fingerprint.clone(),
        attempts: item.attempts.to_string(),
        notes: text::truncate(&item.notes, 64),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_profile_views(items: &[ForecastProfile]) -> Vec<ForecastProfileView> {
    let mut views: Vec<ForecastProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_profile_view(item));
    }
    views
}

pub fn sum_forecast_profile_view(view: &ForecastProfileView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_ceiling + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_profile("code-904", "name-753");
        let view = to_forecast_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
