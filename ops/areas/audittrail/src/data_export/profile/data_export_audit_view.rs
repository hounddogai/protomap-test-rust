use super::*;

use acme_kit::text;

/// How screens and exports show a data export audit.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditView {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub checksum_text: String,
    pub sequence_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_data_export_audit_view(item: &DataExportAudit) -> DataExportAuditView {
    DataExportAuditView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        reviewer: item.reviewer.clone(),
        checksum_text: item.checksum.clone(),
        sequence_text: item.sequence.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_audit_views(items: &[DataExportAudit]) -> Vec<DataExportAuditView> {
    let mut views: Vec<DataExportAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_audit_view(item));
    }
    views
}

pub fn sum_data_export_audit_view(view: &DataExportAuditView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_audit("code-520", "title-976");
        let view = to_data_export_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
