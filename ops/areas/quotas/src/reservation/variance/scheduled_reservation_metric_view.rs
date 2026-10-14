use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled reservation metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationMetricView {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub timezone: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_scheduled_reservation_metric_view(item: &ScheduledReservationMetric) -> ScheduledReservationMetricView {
    ScheduledReservationMetricView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 64),
        fingerprint: item.fingerprint.clone(),
        timezone: item.timezone.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_reservation_metric_views(
    items: &[ScheduledReservationMetric],
) -> Vec<ScheduledReservationMetricView> {
    let mut views: Vec<ScheduledReservationMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_reservation_metric_view(item));
    }
    views
}

pub fn sum_scheduled_reservation_metric_view(view: &ScheduledReservationMetricView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_reservation_metric("code-685", "label-687");
        let view = to_scheduled_reservation_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_reservation_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
