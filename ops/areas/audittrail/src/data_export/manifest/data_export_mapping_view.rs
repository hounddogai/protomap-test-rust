use super::*;

use acme_kit::text;

/// How screens and exports show a data export mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMappingView {
    pub code: String,
    pub label_text: String,
    pub expires_at: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_data_export_mapping_view(item: &DataExportMapping) -> DataExportMappingView {
    DataExportMappingView {
        code: text::truncate(&item.code, 64),
        label_text: text::truncate(&item.label, 120),
        expires_at: item.expires_at.to_string(),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_mapping_views(items: &[DataExportMapping]) -> Vec<DataExportMappingView> {
    let mut views: Vec<DataExportMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_mapping_view(item));
    }
    views
}

pub fn sum_data_export_mapping_view(view: &DataExportMappingView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_mapping("code-304", "label-36");
        let view = to_data_export_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
