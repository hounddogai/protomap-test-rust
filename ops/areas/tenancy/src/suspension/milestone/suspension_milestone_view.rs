use super::*;

/// How screens and exports show a suspension milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestoneView {
    pub code: String,
    pub label: String,
    pub discount_text: String,
    pub burst: String,
    pub phone: String,
    pub capacity: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_suspension_milestone_view(item: &SuspensionMilestone) -> SuspensionMilestoneView {
    SuspensionMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        discount_text: format!("{:.2}", item.discount),
        burst: item.burst.to_string(),
        phone: item.phone.clone(),
        capacity: item.capacity.to_string(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_milestone_views(items: &[SuspensionMilestone]) -> Vec<SuspensionMilestoneView> {
    let mut views: Vec<SuspensionMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_milestone_view(item));
    }
    views
}

pub fn sum_suspension_milestone_view(view: &SuspensionMilestoneView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_discount + (parsed_burst as f64) + (parsed_capacity as f64))
}
