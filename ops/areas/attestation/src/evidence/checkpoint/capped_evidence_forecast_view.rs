use super::*;

/// How screens and exports show a capped evidence forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceForecastView {
    pub code_text: String,
    pub name: String,
    pub archived: String,
    pub version: String,
    pub ends_at: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_capped_evidence_forecast_view(item: &CappedEvidenceForecast) -> CappedEvidenceForecastView {
    CappedEvidenceForecastView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        version: item.version.to_string(),
        ends_at: item.ends_at.to_string(),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_evidence_forecast_views(items: &[CappedEvidenceForecast]) -> Vec<CappedEvidenceForecastView> {
    let mut views: Vec<CappedEvidenceForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_evidence_forecast_view(item));
    }
    views
}

pub fn sum_capped_evidence_forecast_view(view: &CappedEvidenceForecastView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_evidence_forecast("code-506", "name-918");
        let view = to_capped_evidence_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_evidence_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
