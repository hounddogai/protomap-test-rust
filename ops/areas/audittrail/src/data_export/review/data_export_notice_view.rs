use super::*;

use acme_kit::text;

/// How screens and exports show a data export notice.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportNoticeView {
    pub code: String,
    pub label: String,
    pub country: String,
    pub principal: String,
    pub tenant_id: String,
    pub summary: String,
    pub scope_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_data_export_notice_view(item: &DataExportNotice) -> DataExportNoticeView {
    DataExportNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        country: item.country.clone(),
        principal: text::truncate(&item.principal, 120),
        tenant_id: item.tenant_id.clone(),
        summary: text::truncate(&item.summary, 40),
        scope_text: item.scope.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_notice_views(items: &[DataExportNotice]) -> Vec<DataExportNoticeView> {
    let mut views: Vec<DataExportNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_notice_view(item));
    }
    views
}

pub fn sum_data_export_notice_view(view: &DataExportNoticeView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_notice("code-172", "label-498");
        let view = to_data_export_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
