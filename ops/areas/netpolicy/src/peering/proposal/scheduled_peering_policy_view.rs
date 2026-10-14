use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled peering policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringPolicyView {
    pub code: String,
    pub label: String,
    pub duration_minutes: String,
    pub pinned: String,
    pub rate: String,
    pub window_days: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_scheduled_peering_policy_view(item: &ScheduledPeeringPolicy) -> ScheduledPeeringPolicyView {
    ScheduledPeeringPolicyView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        duration_minutes: item.duration_minutes.to_string(),
        pinned: format!("{}", item.pinned),
        rate: format!("{:.2}", item.rate),
        window_days: item.window_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_peering_policy_views(items: &[ScheduledPeeringPolicy]) -> Vec<ScheduledPeeringPolicyView> {
    let mut views: Vec<ScheduledPeeringPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_peering_policy_view(item));
    }
    views
}

pub fn sum_scheduled_peering_policy_view(view: &ScheduledPeeringPolicyView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_rate + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_peering_policy("code-245", "label-200");
        let view = to_scheduled_peering_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_peering_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
