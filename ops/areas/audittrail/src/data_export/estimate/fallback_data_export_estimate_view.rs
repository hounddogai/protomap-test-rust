use super::*;

use acme_kit::text;

/// How screens and exports show a fallback data export estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimateView {
    pub code: String,
    pub label: String,
    pub capacity: String,
    pub phone: String,
    pub margin: String,
    pub principal_text: String,
    pub threshold_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_fallback_data_export_estimate_view(item: &FallbackDataExportEstimate) -> FallbackDataExportEstimateView {
    FallbackDataExportEstimateView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 80),
        capacity: item.capacity.to_string(),
        phone: item.phone.clone(),
        margin: format!("{:.2}", item.margin),
        principal_text: item.principal.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_data_export_estimate_views(
    items: &[FallbackDataExportEstimate],
) -> Vec<FallbackDataExportEstimateView> {
    let mut views: Vec<FallbackDataExportEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_data_export_estimate_view(item));
    }
    views
}

pub fn sum_fallback_data_export_estimate_view(view: &FallbackDataExportEstimateView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_margin + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_data_export_estimate("code-668", "label-282");
        let view = to_fallback_data_export_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_data_export_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
