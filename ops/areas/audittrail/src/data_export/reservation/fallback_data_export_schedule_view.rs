use super::*;

use acme_kit::text;

/// How screens and exports show a fallback data export schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportScheduleView {
    pub code: String,
    pub label: String,
    pub starts_at_text: String,
    pub segment_text: String,
    pub currency_text: String,
    pub timezone: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_fallback_data_export_schedule_view(item: &FallbackDataExportSchedule) -> FallbackDataExportScheduleView {
    FallbackDataExportScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        starts_at_text: item.starts_at.to_string(),
        segment_text: item.segment.clone(),
        currency_text: item.currency.clone(),
        timezone: text::truncate(&item.timezone, 120),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_data_export_schedule_views(
    items: &[FallbackDataExportSchedule],
) -> Vec<FallbackDataExportScheduleView> {
    let mut views: Vec<FallbackDataExportScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_data_export_schedule_view(item));
    }
    views
}

pub fn sum_fallback_data_export_schedule_view(view: &FallbackDataExportScheduleView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_data_export_schedule("code-338", "label-285");
        let view = to_fallback_data_export_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_data_export_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
