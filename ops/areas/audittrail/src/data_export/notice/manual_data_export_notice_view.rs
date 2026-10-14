use super::*;

use acme_kit::text;

/// How screens and exports show a manual data export notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNoticeView {
    pub code: String,
    pub label_text: String,
    pub amount: String,
    pub title: String,
    pub visible: String,
    pub starts_at: String,
    pub phone: String,
    pub sequence_text: String,
    pub key_id_text: String,
    pub contact_email: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_manual_data_export_notice_view(item: &ManualDataExportNotice) -> ManualDataExportNoticeView {
    ManualDataExportNoticeView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        title: item.title.clone(),
        visible: format!("{}", item.visible),
        starts_at: item.starts_at.to_string(),
        phone: item.phone.clone(),
        sequence_text: item.sequence.to_string(),
        key_id_text: text::truncate(&item.key_id, 64),
        contact_email: item.contact_email.clone(),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_data_export_notice_views(items: &[ManualDataExportNotice]) -> Vec<ManualDataExportNoticeView> {
    let mut views: Vec<ManualDataExportNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_data_export_notice_view(item));
    }
    views
}

pub fn sum_manual_data_export_notice_view(view: &ManualDataExportNoticeView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_amount + (parsed_starts_at as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_data_export_notice("code-230", "label-370");
        let view = to_manual_data_export_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_data_export_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
