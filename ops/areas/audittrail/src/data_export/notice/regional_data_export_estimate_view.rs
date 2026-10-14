use super::*;

use acme_kit::text;

/// How screens and exports show a regional data export estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportEstimateView {
    pub code: String,
    pub label: String,
    pub timezone_text: String,
    pub category: String,
    pub starts_at: String,
    pub quota_units_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_regional_data_export_estimate_view(item: &RegionalDataExportEstimate) -> RegionalDataExportEstimateView {
    RegionalDataExportEstimateView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 120),
        timezone_text: item.timezone.clone(),
        category: text::truncate(&item.category, 64),
        starts_at: item.starts_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_data_export_estimate_views(
    items: &[RegionalDataExportEstimate],
) -> Vec<RegionalDataExportEstimateView> {
    let mut views: Vec<RegionalDataExportEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_data_export_estimate_view(item));
    }
    views
}

pub fn sum_regional_data_export_estimate_view(view: &RegionalDataExportEstimateView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_data_export_estimate("code-604", "label-361");
        let view = to_regional_data_export_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_data_export_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
