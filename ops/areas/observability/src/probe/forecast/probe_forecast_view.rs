use super::*;

use acme_kit::text;

/// How screens and exports show a probe forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeForecastView {
    pub code_text: String,
    pub label_text: String,
    pub attempts: String,
    pub priority_text: String,
    pub timezone_text: String,
    pub summary: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_probe_forecast_view(item: &ProbeForecast) -> ProbeForecastView {
    ProbeForecastView {
        code_text: text::truncate(&item.code, 40),
        label_text: text::truncate(&item.label, 120),
        attempts: item.attempts.to_string(),
        priority_text: item.priority.to_string(),
        timezone_text: item.timezone.clone(),
        summary: item.summary.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_forecast_views(items: &[ProbeForecast]) -> Vec<ProbeForecastView> {
    let mut views: Vec<ProbeForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_forecast_view(item));
    }
    views
}

pub fn sum_probe_forecast_view(view: &ProbeForecastView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_priority as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_forecast("code-752", "label-94");
        let view = to_probe_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
