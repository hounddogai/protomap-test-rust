use super::*;

use acme_kit::text;

/// How screens and exports show a health check milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMilestoneView {
    pub code: String,
    pub label_text: String,
    pub key_id: String,
    pub quantity: String,
    pub priority_text: String,
    pub starts_at: String,
    pub timezone: String,
    pub fingerprint: String,
    pub segment_text: String,
    pub region: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_health_check_milestone_view(item: &HealthCheckMilestone) -> HealthCheckMilestoneView {
    HealthCheckMilestoneView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        key_id: item.key_id.clone(),
        quantity: item.quantity.to_string(),
        priority_text: item.priority.to_string(),
        starts_at: item.starts_at.to_string(),
        timezone: item.timezone.clone(),
        fingerprint: item.fingerprint.clone(),
        segment_text: text::truncate(&item.segment, 80),
        region: item.region.clone(),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_milestone_views(items: &[HealthCheckMilestone]) -> Vec<HealthCheckMilestoneView> {
    let mut views: Vec<HealthCheckMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_milestone_view(item));
    }
    views
}

pub fn sum_health_check_milestone_view(view: &HealthCheckMilestoneView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_priority as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_milestone("code-871", "label-402");
        let view = to_health_check_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
