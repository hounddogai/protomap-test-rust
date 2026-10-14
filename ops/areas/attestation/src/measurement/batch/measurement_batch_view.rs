use super::*;

use acme_kit::text;

/// How screens and exports show a measurement batch.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBatchView {
    pub code: String,
    pub title: String,
    pub batch_size: String,
    pub currency_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_measurement_batch_view(item: &MeasurementBatch) -> MeasurementBatchView {
    MeasurementBatchView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        batch_size: item.batch_size.to_string(),
        currency_text: text::truncate(&item.currency, 40),
        tenant_id: text::truncate(&item.tenant_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_batch_views(items: &[MeasurementBatch]) -> Vec<MeasurementBatchView> {
    let mut views: Vec<MeasurementBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_batch_view(item));
    }
    views
}

pub fn sum_measurement_batch_view(view: &MeasurementBatchView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_batch("code-67", "title-459");
        let view = to_measurement_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
