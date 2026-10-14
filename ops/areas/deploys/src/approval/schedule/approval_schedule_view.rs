use super::*;

use acme_kit::text;

/// How screens and exports show a approval schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScheduleView {
    pub code: String,
    pub label: String,
    pub name_text: String,
    pub amount_text: String,
    pub fingerprint: String,
    pub unit: String,
    pub country_text: String,
    pub quantity: String,
    pub reviewer: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_approval_schedule_view(item: &ApprovalSchedule) -> ApprovalScheduleView {
    ApprovalScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        name_text: item.name.clone(),
        amount_text: format!("{:.2}", item.amount),
        fingerprint: item.fingerprint.clone(),
        unit: item.unit.clone(),
        country_text: item.country.clone(),
        quantity: item.quantity.to_string(),
        reviewer: text::truncate(&item.reviewer, 64),
        host_text: text::truncate(&item.host, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_schedule_views(items: &[ApprovalSchedule]) -> Vec<ApprovalScheduleView> {
    let mut views: Vec<ApprovalScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_schedule_view(item));
    }
    views
}

pub fn sum_approval_schedule_view(view: &ApprovalScheduleView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_amount + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_schedule("code-211", "label-763");
        let view = to_approval_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
