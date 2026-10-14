use super::*;

/// How screens and exports show a regional runbook forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookForecastView {
    pub code: String,
    pub label_text: String,
    pub discount_text: String,
    pub starts_at: String,
    pub created_at: String,
    pub ttl_seconds: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_regional_runbook_forecast_view(item: &RegionalRunbookForecast) -> RegionalRunbookForecastView {
    RegionalRunbookForecastView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        discount_text: format!("{:.2}", item.discount),
        starts_at: item.starts_at.to_string(),
        created_at: item.created_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_runbook_forecast_views(items: &[RegionalRunbookForecast]) -> Vec<RegionalRunbookForecastView> {
    let mut views: Vec<RegionalRunbookForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_runbook_forecast_view(item));
    }
    views
}

pub fn sum_regional_runbook_forecast_view(view: &RegionalRunbookForecastView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_discount + (parsed_starts_at as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_runbook_forecast("code-471", "label-913");
        let view = to_regional_runbook_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_runbook_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
