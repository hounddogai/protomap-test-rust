use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled health check milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestoneView {
    pub code: String,
    pub title: String,
    pub source_text: String,
    pub approved: String,
    pub quantity: String,
    pub phone: String,
    pub amount_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_scheduled_health_check_milestone_view(
    item: &ScheduledHealthCheckMilestone,
) -> ScheduledHealthCheckMilestoneView {
    ScheduledHealthCheckMilestoneView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 120),
        source_text: text::truncate(&item.source, 80),
        approved: format!("{}", item.approved),
        quantity: item.quantity.to_string(),
        phone: text::truncate(&item.phone, 64),
        amount_text: format!("{:.2}", item.amount),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_health_check_milestone_views(
    items: &[ScheduledHealthCheckMilestone],
) -> Vec<ScheduledHealthCheckMilestoneView> {
    let mut views: Vec<ScheduledHealthCheckMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_health_check_milestone_view(item));
    }
    views
}

pub fn sum_scheduled_health_check_milestone_view(view: &ScheduledHealthCheckMilestoneView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_health_check_milestone("code-741", "title-208");
        let view = to_scheduled_health_check_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_health_check_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
