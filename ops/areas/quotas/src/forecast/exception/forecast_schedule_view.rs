use super::*;

use acme_kit::text;

/// How screens and exports show a forecast schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScheduleView {
    pub code_text: String,
    pub name: String,
    pub window_days_text: String,
    pub revision: String,
    pub region: String,
    pub fingerprint: String,
    pub country_text: String,
    pub external_ref: String,
    pub minimum: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_forecast_schedule_view(item: &ForecastSchedule) -> ForecastScheduleView {
    ForecastScheduleView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        window_days_text: item.window_days.to_string(),
        revision: item.revision.to_string(),
        region: item.region.clone(),
        fingerprint: item.fingerprint.clone(),
        country_text: item.country.clone(),
        external_ref: item.external_ref.clone(),
        minimum: format!("{:.2}", item.minimum),
        tenant_id: text::truncate(&item.tenant_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_schedule_views(items: &[ForecastSchedule]) -> Vec<ForecastScheduleView> {
    let mut views: Vec<ForecastScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_schedule_view(item));
    }
    views
}

pub fn sum_forecast_schedule_view(view: &ForecastScheduleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_revision as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_schedule("code-997", "name-928");
        let view = to_forecast_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
