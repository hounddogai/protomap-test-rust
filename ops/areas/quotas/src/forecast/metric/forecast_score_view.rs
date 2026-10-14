use super::*;

use acme_kit::text;

/// How screens and exports show a forecast score.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScoreView {
    pub code_text: String,
    pub label_text: String,
    pub revision_text: String,
    pub rate_text: String,
    pub source: String,
    pub floor_value_text: String,
    pub locale: String,
    pub ceiling_text: String,
    pub priority_text: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_forecast_score_view(item: &ForecastScore) -> ForecastScoreView {
    ForecastScoreView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        revision_text: item.revision.to_string(),
        rate_text: format!("{:.2}", item.rate),
        source: item.source.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        locale: item.locale.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        priority_text: item.priority.to_string(),
        host_text: text::truncate(&item.host, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_score_views(items: &[ForecastScore]) -> Vec<ForecastScoreView> {
    let mut views: Vec<ForecastScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_score_view(item));
    }
    views
}

pub fn sum_forecast_score_view(view: &ForecastScoreView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_revision as f64 + parsed_rate + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_score("code-156", "label-199");
        let view = to_forecast_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
