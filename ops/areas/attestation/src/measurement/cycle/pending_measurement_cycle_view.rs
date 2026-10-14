use super::*;

use acme_kit::text;

/// How screens and exports show a pending measurement cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementCycleView {
    pub code_text: String,
    pub label_text: String,
    pub updated_at: String,
    pub expires_at_text: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_pending_measurement_cycle_view(item: &PendingMeasurementCycle) -> PendingMeasurementCycleView {
    PendingMeasurementCycleView {
        code_text: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        expires_at_text: item.expires_at.to_string(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_measurement_cycle_views(items: &[PendingMeasurementCycle]) -> Vec<PendingMeasurementCycleView> {
    let mut views: Vec<PendingMeasurementCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_measurement_cycle_view(item));
    }
    views
}

pub fn sum_pending_measurement_cycle_view(view: &PendingMeasurementCycleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_measurement_cycle("code-495", "label-413");
        let view = to_pending_measurement_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_measurement_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
