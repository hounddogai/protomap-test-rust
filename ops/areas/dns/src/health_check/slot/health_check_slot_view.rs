use super::*;

use acme_kit::text;

/// How screens and exports show a health check slot.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlotView {
    pub code: String,
    pub title: String,
    pub grace_days_text: String,
    pub notes: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_health_check_slot_view(item: &HealthCheckSlot) -> HealthCheckSlotView {
    HealthCheckSlotView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        grace_days_text: item.grace_days.to_string(),
        notes: item.notes.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_slot_views(items: &[HealthCheckSlot]) -> Vec<HealthCheckSlotView> {
    let mut views: Vec<HealthCheckSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_slot_view(item));
    }
    views
}

pub fn sum_health_check_slot_view(view: &HealthCheckSlotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_slot("code-854", "title-739");
        let view = to_health_check_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
