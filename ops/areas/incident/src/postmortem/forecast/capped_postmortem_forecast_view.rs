use super::*;

use acme_kit::text;

/// How screens and exports show a capped postmortem forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemForecastView {
    pub code: String,
    pub label: String,
    pub revision: String,
    pub quantity_text: String,
    pub threshold: String,
    pub namespace: String,
    pub discount: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_capped_postmortem_forecast_view(item: &CappedPostmortemForecast) -> CappedPostmortemForecastView {
    CappedPostmortemForecastView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        revision: item.revision.to_string(),
        quantity_text: item.quantity.to_string(),
        threshold: format!("{:.2}", item.threshold),
        namespace: item.namespace.clone(),
        discount: format!("{:.2}", item.discount),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_postmortem_forecast_views(items: &[CappedPostmortemForecast]) -> Vec<CappedPostmortemForecastView> {
    let mut views: Vec<CappedPostmortemForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_postmortem_forecast_view(item));
    }
    views
}

pub fn sum_capped_postmortem_forecast_view(view: &CappedPostmortemForecastView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quantity as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_postmortem_forecast("code-448", "label-727");
        let view = to_capped_postmortem_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_postmortem_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
