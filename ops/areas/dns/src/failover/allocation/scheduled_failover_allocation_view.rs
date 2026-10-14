use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled failover allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverAllocationView {
    pub code: String,
    pub label: String,
    pub source_text: String,
    pub attempts: String,
    pub minimum: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_scheduled_failover_allocation_view(item: &ScheduledFailoverAllocation) -> ScheduledFailoverAllocationView {
    ScheduledFailoverAllocationView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        source_text: item.source.clone(),
        attempts: item.attempts.to_string(),
        minimum: format!("{:.2}", item.minimum),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_failover_allocation_views(
    items: &[ScheduledFailoverAllocation],
) -> Vec<ScheduledFailoverAllocationView> {
    let mut views: Vec<ScheduledFailoverAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_failover_allocation_view(item));
    }
    views
}

pub fn sum_scheduled_failover_allocation_view(view: &ScheduledFailoverAllocationView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_minimum + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_failover_allocation("code-190", "label-50");
        let view = to_scheduled_failover_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_failover_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
