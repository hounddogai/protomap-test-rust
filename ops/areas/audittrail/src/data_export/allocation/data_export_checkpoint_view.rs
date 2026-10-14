use super::*;

use acme_kit::text;

/// How screens and exports show a data export checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpointView {
    pub code: String,
    pub title: String,
    pub quota_units: String,
    pub quantity: String,
    pub namespace_text: String,
    pub rotation_days: String,
    pub budget_text: String,
    pub minimum: String,
    pub fingerprint: String,
    pub duration_minutes: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_data_export_checkpoint_view(item: &DataExportCheckpoint) -> DataExportCheckpointView {
    DataExportCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        quota_units: item.quota_units.to_string(),
        quantity: item.quantity.to_string(),
        namespace_text: text::truncate(&item.namespace, 40),
        rotation_days: item.rotation_days.to_string(),
        budget_text: format!("{:.2}", item.budget),
        minimum: format!("{:.2}", item.minimum),
        fingerprint: item.fingerprint.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_checkpoint_views(items: &[DataExportCheckpoint]) -> Vec<DataExportCheckpointView> {
    let mut views: Vec<DataExportCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_checkpoint_view(item));
    }
    views
}

pub fn sum_data_export_checkpoint_view(view: &DataExportCheckpointView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_quantity as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_checkpoint("code-265", "title-290");
        let view = to_data_export_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
