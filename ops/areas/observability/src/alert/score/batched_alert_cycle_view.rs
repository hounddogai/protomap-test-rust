use super::*;

use acme_kit::text;

/// How screens and exports show a batched alert cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycleView {
    pub code_text: String,
    pub label_text: String,
    pub segment: String,
    pub channel: String,
    pub tenant_id: String,
    pub unit: String,
    pub starts_at_text: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_batched_alert_cycle_view(item: &BatchedAlertCycle) -> BatchedAlertCycleView {
    BatchedAlertCycleView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        segment: item.segment.clone(),
        channel: text::truncate(&item.channel, 64),
        tenant_id: item.tenant_id.clone(),
        unit: text::truncate(&item.unit, 64),
        starts_at_text: item.starts_at.to_string(),
        notes_text: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_alert_cycle_views(items: &[BatchedAlertCycle]) -> Vec<BatchedAlertCycleView> {
    let mut views: Vec<BatchedAlertCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_alert_cycle_view(item));
    }
    views
}

pub fn sum_batched_alert_cycle_view(view: &BatchedAlertCycleView) -> Result<f64, String> {
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
        let item = new_batched_alert_cycle("code-905", "label-944");
        let view = to_batched_alert_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_alert_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
