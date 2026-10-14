use super::*;

use acme_kit::text;

/// How screens and exports show a legacy measurement batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBatchView {
    pub code_text: String,
    pub name: String,
    pub phone: String,
    pub summary: String,
    pub floor_value_text: String,
    pub minimum: String,
    pub version: String,
    pub approved: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_legacy_measurement_batch_view(item: &LegacyMeasurementBatch) -> LegacyMeasurementBatchView {
    LegacyMeasurementBatchView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        phone: text::truncate(&item.phone, 64),
        summary: item.summary.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        minimum: format!("{:.2}", item.minimum),
        version: item.version.to_string(),
        approved: format!("{}", item.approved),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_measurement_batch_views(items: &[LegacyMeasurementBatch]) -> Vec<LegacyMeasurementBatchView> {
    let mut views: Vec<LegacyMeasurementBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_measurement_batch_view(item));
    }
    views
}

pub fn sum_legacy_measurement_batch_view(view: &LegacyMeasurementBatchView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_floor_value + parsed_minimum + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_measurement_batch("code-14", "name-809");
        let view = to_legacy_measurement_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_measurement_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
