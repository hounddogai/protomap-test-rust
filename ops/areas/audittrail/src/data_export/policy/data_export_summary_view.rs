use super::*;

use acme_kit::text;

/// How screens and exports show a data export summary.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSummaryView {
    pub code: String,
    pub name: String,
    pub active: String,
    pub maximum: String,
    pub revision: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_data_export_summary_view(item: &DataExportSummary) -> DataExportSummaryView {
    DataExportSummaryView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        active: format!("{}", item.active),
        maximum: format!("{:.2}", item.maximum),
        revision: item.revision.to_string(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_summary_views(items: &[DataExportSummary]) -> Vec<DataExportSummaryView> {
    let mut views: Vec<DataExportSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_summary_view(item));
    }
    views
}

pub fn sum_data_export_summary_view(view: &DataExportSummaryView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_maximum + (parsed_revision as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_summary("code-88", "name-527");
        let view = to_data_export_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
