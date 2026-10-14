use super::*;

/// How screens and exports show a manual data export checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpointView {
    pub code: String,
    pub name: String,
    pub priority: String,
    pub owner_id_text: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_manual_data_export_checkpoint_view(item: &ManualDataExportCheckpoint) -> ManualDataExportCheckpointView {
    ManualDataExportCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        priority: item.priority.to_string(),
        owner_id_text: item.owner_id.clone(),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_data_export_checkpoint_views(
    items: &[ManualDataExportCheckpoint],
) -> Vec<ManualDataExportCheckpointView> {
    let mut views: Vec<ManualDataExportCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_data_export_checkpoint_view(item));
    }
    views
}

pub fn sum_manual_data_export_checkpoint_view(view: &ManualDataExportCheckpointView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_data_export_checkpoint("code-596", "name-536");
        let view = to_manual_data_export_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_data_export_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
