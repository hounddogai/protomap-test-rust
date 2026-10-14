use super::*;

/// How screens and exports show a scheduled data export summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportSummaryView {
    pub code_text: String,
    pub label: String,
    pub maximum_text: String,
    pub summary: String,
    pub name: String,
    pub discount: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_scheduled_data_export_summary_view(item: &ScheduledDataExportSummary) -> ScheduledDataExportSummaryView {
    ScheduledDataExportSummaryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        summary: item.summary.clone(),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_data_export_summary_views(
    items: &[ScheduledDataExportSummary],
) -> Vec<ScheduledDataExportSummaryView> {
    let mut views: Vec<ScheduledDataExportSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_data_export_summary_view(item));
    }
    views
}

pub fn sum_scheduled_data_export_summary_view(view: &ScheduledDataExportSummaryView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_maximum + parsed_discount + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_data_export_summary("code-356", "label-666");
        let view = to_scheduled_data_export_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_data_export_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
