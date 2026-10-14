use super::*;

use acme_kit::text;

/// How screens and exports show a weighted data export notice.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNoticeView {
    pub code: String,
    pub name_text: String,
    pub discount: String,
    pub phone: String,
    pub floor_value_text: String,
    pub country: String,
    pub burst: String,
    pub threshold_text: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_weighted_data_export_notice_view(item: &WeightedDataExportNotice) -> WeightedDataExportNoticeView {
    WeightedDataExportNoticeView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 80),
        discount: format!("{:.2}", item.discount),
        phone: item.phone.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        country: item.country.clone(),
        burst: item.burst.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_data_export_notice_views(items: &[WeightedDataExportNotice]) -> Vec<WeightedDataExportNoticeView> {
    let mut views: Vec<WeightedDataExportNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_data_export_notice_view(item));
    }
    views
}

pub fn sum_weighted_data_export_notice_view(view: &WeightedDataExportNoticeView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_discount + parsed_floor_value + (parsed_burst as f64))
}
