use super::*;

use acme_kit::text;

/// How screens and exports show a capped forecast score.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastScoreView {
    pub code: String,
    pub name: String,
    pub host: String,
    pub notes: String,
    pub tenant_id: String,
    pub discount: String,
    pub grace_days: String,
    pub version: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_capped_forecast_score_view(item: &CappedForecastScore) -> CappedForecastScoreView {
    CappedForecastScoreView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        host: item.host.clone(),
        notes: item.notes.clone(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        discount: format!("{:.2}", item.discount),
        grace_days: item.grace_days.to_string(),
        version: item.version.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_forecast_score_views(items: &[CappedForecastScore]) -> Vec<CappedForecastScoreView> {
    let mut views: Vec<CappedForecastScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_forecast_score_view(item));
    }
    views
}

pub fn sum_capped_forecast_score_view(view: &CappedForecastScoreView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_discount + (parsed_grace_days as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_forecast_score("code-484", "name-79");
        let view = to_capped_forecast_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_forecast_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
