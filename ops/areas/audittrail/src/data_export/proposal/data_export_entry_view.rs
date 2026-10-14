use super::*;

use acme_kit::text;

/// How screens and exports show a data export entry.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryView {
    pub code: String,
    pub name_text: String,
    pub updated_at_text: String,
    pub phone_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_data_export_entry_view(item: &DataExportEntry) -> DataExportEntryView {
    DataExportEntryView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        updated_at_text: item.updated_at.to_string(),
        phone_text: item.phone.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_entry_views(items: &[DataExportEntry]) -> Vec<DataExportEntryView> {
    let mut views: Vec<DataExportEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_entry_view(item));
    }
    views
}

pub fn sum_data_export_entry_view(view: &DataExportEntryView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_entry("code-794", "name-524");
        let view = to_data_export_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
