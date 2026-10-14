use super::*;

use acme_kit::text;

/// How screens and exports show a regional forecast bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastBundleView {
    pub code: String,
    pub label_text: String,
    pub approved: String,
    pub ttl_seconds: String,
    pub version: String,
    pub reviewer_text: String,
    pub external_ref_text: String,
    pub phone: String,
    pub region: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_regional_forecast_bundle_view(item: &RegionalForecastBundle) -> RegionalForecastBundleView {
    RegionalForecastBundleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        approved: format!("{}", item.approved),
        ttl_seconds: item.ttl_seconds.to_string(),
        version: item.version.to_string(),
        reviewer_text: item.reviewer.clone(),
        external_ref_text: item.external_ref.clone(),
        phone: text::truncate(&item.phone, 80),
        region: item.region.clone(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_forecast_bundle_views(items: &[RegionalForecastBundle]) -> Vec<RegionalForecastBundleView> {
    let mut views: Vec<RegionalForecastBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_forecast_bundle_view(item));
    }
    views
}

pub fn sum_regional_forecast_bundle_view(view: &RegionalForecastBundleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_forecast_bundle("code-334", "label-677");
        let view = to_regional_forecast_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_forecast_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
