use super::*;

/// How screens and exports show a scheduled limit forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitForecastView {
    pub code: String,
    pub name: String,
    pub attempts: String,
    pub duration_minutes: String,
    pub region_text: String,
    pub key_id: String,
    pub pinned: String,
    pub currency: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_scheduled_limit_forecast_view(item: &ScheduledLimitForecast) -> ScheduledLimitForecastView {
    ScheduledLimitForecastView {
        code: item.code.clone(),
        name: item.name.clone(),
        attempts: item.attempts.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        region_text: item.region.clone(),
        key_id: item.key_id.clone(),
        pinned: format!("{}", item.pinned),
        currency: item.currency.clone(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_limit_forecast_views(items: &[ScheduledLimitForecast]) -> Vec<ScheduledLimitForecastView> {
    let mut views: Vec<ScheduledLimitForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_limit_forecast_view(item));
    }
    views
}

pub fn sum_scheduled_limit_forecast_view(view: &ScheduledLimitForecastView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_limit_forecast("code-448", "name-477");
        let view = to_scheduled_limit_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_limit_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
