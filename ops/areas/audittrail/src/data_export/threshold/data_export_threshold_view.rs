use super::*;

use acme_kit::text;

/// How screens and exports show a data export threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdView {
    pub code_text: String,
    pub label: String,
    pub approved_text: String,
    pub cost: String,
    pub checksum: String,
    pub created_at_text: String,
    pub fingerprint: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_data_export_threshold_view(item: &DataExportThreshold) -> DataExportThresholdView {
    DataExportThresholdView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 120),
        approved_text: format!("{}", item.approved),
        cost: format!("{:.2}", item.cost),
        checksum: text::truncate(&item.checksum, 80),
        created_at_text: item.created_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_threshold_views(items: &[DataExportThreshold]) -> Vec<DataExportThresholdView> {
    let mut views: Vec<DataExportThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_threshold_view(item));
    }
    views
}

pub fn sum_data_export_threshold_view(view: &DataExportThresholdView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_cost + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_threshold("code-316", "label-543");
        let view = to_data_export_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
