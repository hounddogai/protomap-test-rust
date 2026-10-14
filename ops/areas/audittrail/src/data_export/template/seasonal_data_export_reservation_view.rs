use super::*;

/// How screens and exports show a seasonal data export reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservationView {
    pub code: String,
    pub label_text: String,
    pub threshold: String,
    pub window_days: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_seasonal_data_export_reservation_view(
    item: &SeasonalDataExportReservation,
) -> SeasonalDataExportReservationView {
    SeasonalDataExportReservationView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        window_days: item.window_days.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_data_export_reservation_views(
    items: &[SeasonalDataExportReservation],
) -> Vec<SeasonalDataExportReservationView> {
    let mut views: Vec<SeasonalDataExportReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_data_export_reservation_view(item));
    }
    views
}

pub fn sum_seasonal_data_export_reservation_view(view: &SeasonalDataExportReservationView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_threshold + (parsed_window_days as f64) + (parsed_batch_size as f64))
}
