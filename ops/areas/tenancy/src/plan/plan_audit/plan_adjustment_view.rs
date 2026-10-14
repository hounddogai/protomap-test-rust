use super::*;

use acme_kit::text;

/// How screens and exports show a plan adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustmentView {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub port: String,
    pub host: String,
    pub visible: String,
    pub priority_text: String,
    pub contact_email_text: String,
    pub description_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_plan_adjustment_view(item: &PlanAdjustment) -> PlanAdjustmentView {
    PlanAdjustmentView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        key_id: item.key_id.clone(),
        port: item.port.to_string(),
        host: item.host.clone(),
        visible: format!("{}", item.visible),
        priority_text: item.priority.to_string(),
        contact_email_text: text::truncate(&item.contact_email, 40),
        description_text: item.description.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_adjustment_views(items: &[PlanAdjustment]) -> Vec<PlanAdjustmentView> {
    let mut views: Vec<PlanAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_adjustment_view(item));
    }
    views
}

pub fn sum_plan_adjustment_view(view: &PlanAdjustmentView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_port as f64 + (parsed_priority as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_adjustment("code-702", "title-6");
        let view = to_plan_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
