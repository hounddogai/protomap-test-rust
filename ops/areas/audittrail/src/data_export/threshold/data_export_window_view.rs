use super::*;

use acme_kit::text;

/// How screens and exports show a data export window.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportWindowView {
    pub code: String,
    pub name: String,
    pub quantity_text: String,
    pub cost_text: String,
    pub contact_email: String,
    pub capacity: String,
    pub channel_text: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_data_export_window_view(item: &DataExportWindow) -> DataExportWindowView {
    DataExportWindowView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        quantity_text: item.quantity.to_string(),
        cost_text: format!("{:.2}", item.cost),
        contact_email: item.contact_email.clone(),
        capacity: item.capacity.to_string(),
        channel_text: item.channel.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_window_views(items: &[DataExportWindow]) -> Vec<DataExportWindowView> {
    let mut views: Vec<DataExportWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_window_view(item));
    }
    views
}

pub fn sum_data_export_window_view(view: &DataExportWindowView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_cost + (parsed_capacity as f64))
}
