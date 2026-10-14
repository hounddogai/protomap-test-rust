use super::*;

use acme_kit::text;

/// How screens and exports show a probe allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAllocationView {
    pub code: String,
    pub label: String,
    pub budget_text: String,
    pub title: String,
    pub attempts: String,
    pub fingerprint: String,
    pub ttl_seconds_text: String,
    pub scope: String,
    pub segment_text: String,
    pub grace_days_text: String,
    pub description_text: String,
    pub status_label: String,
}

pub fn to_probe_allocation_view(item: &ProbeAllocation) -> ProbeAllocationView {
    ProbeAllocationView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        budget_text: format!("{:.2}", item.budget),
        title: text::truncate(&item.title, 120),
        attempts: item.attempts.to_string(),
        fingerprint: item.fingerprint.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        scope: text::truncate(&item.scope, 80),
        segment_text: text::truncate(&item.segment, 40),
        grace_days_text: item.grace_days.to_string(),
        description_text: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_allocation_views(items: &[ProbeAllocation]) -> Vec<ProbeAllocationView> {
    let mut views: Vec<ProbeAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_allocation_view(item));
    }
    views
}

pub fn sum_probe_allocation_view(view: &ProbeAllocationView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_budget + (parsed_attempts as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_allocation("code-414", "label-236");
        let view = to_probe_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
