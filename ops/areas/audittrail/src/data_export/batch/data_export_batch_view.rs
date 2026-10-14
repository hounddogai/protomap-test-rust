use super::*;

/// How screens and exports show a data export batch.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportBatchView {
    pub code_text: String,
    pub label: String,
    pub rotation_days_text: String,
    pub owner_id: String,
    pub created_at: String,
    pub duration_minutes_text: String,
    pub audience: String,
    pub rate_text: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_data_export_batch_view(item: &DataExportBatch) -> DataExportBatchView {
    DataExportBatchView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        owner_id: item.owner_id.clone(),
        created_at: item.created_at.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        audience: item.audience.clone(),
        rate_text: format!("{:.2}", item.rate),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_batch_views(items: &[DataExportBatch]) -> Vec<DataExportBatchView> {
    let mut views: Vec<DataExportBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_batch_view(item));
    }
    views
}

pub fn sum_data_export_batch_view(view: &DataExportBatchView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_created_at as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_batch("code-260", "label-509");
        let view = to_data_export_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
