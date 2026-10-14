use super::*;

use acme_kit::text;

/// How screens and exports show a manual measurement batch.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementBatchView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub expires_at: String,
    pub margin: String,
    pub region_text: String,
    pub cost: String,
    pub rate: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_manual_measurement_batch_view(item: &ManualMeasurementBatch) -> ManualMeasurementBatchView {
    ManualMeasurementBatchView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        port: item.port.to_string(),
        expires_at: item.expires_at.to_string(),
        margin: format!("{:.2}", item.margin),
        region_text: item.region.clone(),
        cost: format!("{:.2}", item.cost),
        rate: format!("{:.2}", item.rate),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_measurement_batch_views(items: &[ManualMeasurementBatch]) -> Vec<ManualMeasurementBatchView> {
    let mut views: Vec<ManualMeasurementBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_measurement_batch_view(item));
    }
    views
}

pub fn sum_manual_measurement_batch_view(view: &ManualMeasurementBatchView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_port as f64 + (parsed_expires_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_measurement_batch("code-876", "label-417");
        let view = to_manual_measurement_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_measurement_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
