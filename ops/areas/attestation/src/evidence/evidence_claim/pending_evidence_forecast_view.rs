use super::*;

use acme_kit::text;

/// How screens and exports show a pending evidence forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEvidenceForecastView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub country: String,
    pub segment: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_pending_evidence_forecast_view(item: &PendingEvidenceForecast) -> PendingEvidenceForecastView {
    PendingEvidenceForecastView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        score: format!("{:.2}", item.score),
        country: item.country.clone(),
        segment: item.segment.clone(),
        principal: text::truncate(&item.principal, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_evidence_forecast_views(items: &[PendingEvidenceForecast]) -> Vec<PendingEvidenceForecastView> {
    let mut views: Vec<PendingEvidenceForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_evidence_forecast_view(item));
    }
    views
}

pub fn sum_pending_evidence_forecast_view(view: &PendingEvidenceForecastView) -> Result<f64, String> {
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
        let item = new_pending_evidence_forecast("code-924", "title-445");
        let view = to_pending_evidence_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_evidence_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
