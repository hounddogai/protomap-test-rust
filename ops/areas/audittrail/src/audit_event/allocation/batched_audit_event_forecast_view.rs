use super::*;

use acme_kit::text;

/// How screens and exports show a batched audit event forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventForecastView {
    pub code_text: String,
    pub name: String,
    pub capacity_text: String,
    pub grace_days_text: String,
    pub approved: String,
    pub ceiling: String,
    pub pinned: String,
    pub namespace_text: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_batched_audit_event_forecast_view(item: &BatchedAuditEventForecast) -> BatchedAuditEventForecastView {
    BatchedAuditEventForecastView {
        code_text: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 80),
        capacity_text: item.capacity.to_string(),
        grace_days_text: item.grace_days.to_string(),
        approved: format!("{}", item.approved),
        ceiling: format!("{:.2}", item.ceiling),
        pinned: format!("{}", item.pinned),
        namespace_text: text::truncate(&item.namespace, 80),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_audit_event_forecast_views(
    items: &[BatchedAuditEventForecast],
) -> Vec<BatchedAuditEventForecastView> {
    let mut views: Vec<BatchedAuditEventForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_audit_event_forecast_view(item));
    }
    views
}

pub fn sum_batched_audit_event_forecast_view(view: &BatchedAuditEventForecastView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_grace_days as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_audit_event_forecast("code-134", "name-641");
        let view = to_batched_audit_event_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_audit_event_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
