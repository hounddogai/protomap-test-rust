use super::*;

/// How screens and exports show a seasonal federation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationExceptionView {
    pub code: String,
    pub label: String,
    pub maximum: String,
    pub country_text: String,
    pub limit: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_seasonal_federation_exception_view(item: &SeasonalFederationException) -> SeasonalFederationExceptionView {
    SeasonalFederationExceptionView {
        code: item.code.clone(),
        label: item.label.clone(),
        maximum: format!("{:.2}", item.maximum),
        country_text: item.country.clone(),
        limit: item.limit.to_string(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_federation_exception_views(
    items: &[SeasonalFederationException],
) -> Vec<SeasonalFederationExceptionView> {
    let mut views: Vec<SeasonalFederationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_federation_exception_view(item));
    }
    views
}

pub fn sum_seasonal_federation_exception_view(view: &SeasonalFederationExceptionView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_maximum + (parsed_limit as f64))
}
