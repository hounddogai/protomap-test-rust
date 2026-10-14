use super::*;

use acme_kit::text;

/// How screens and exports show a pending allowlist forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAllowlistForecastView {
    pub code_text: String,
    pub name: String,
    pub score: String,
    pub archived_text: String,
    pub label: String,
    pub source: String,
    pub tenant_id: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_pending_allowlist_forecast_view(item: &PendingAllowlistForecast) -> PendingAllowlistForecastView {
    PendingAllowlistForecastView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        score: format!("{:.2}", item.score),
        archived_text: format!("{}", item.archived),
        label: item.label.clone(),
        source: text::truncate(&item.source, 64),
        tenant_id: item.tenant_id.clone(),
        category: text::truncate(&item.category, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_allowlist_forecast_views(items: &[PendingAllowlistForecast]) -> Vec<PendingAllowlistForecastView> {
    let mut views: Vec<PendingAllowlistForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_allowlist_forecast_view(item));
    }
    views
}

pub fn sum_pending_allowlist_forecast_view(view: &PendingAllowlistForecastView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_allowlist_forecast("code-478", "name-205");
        let view = to_pending_allowlist_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_allowlist_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
