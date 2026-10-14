use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAdjustmentView {
    pub code_text: String,
    pub title: String,
    pub priority_text: String,
    pub revision: String,
    pub contact_email: String,
    pub checksum: String,
    pub name: String,
    pub ends_at_text: String,
    pub quantity_text: String,
    pub amount: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_dashboard_adjustment_view(item: &DashboardAdjustment) -> DashboardAdjustmentView {
    DashboardAdjustmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        priority_text: item.priority.to_string(),
        revision: item.revision.to_string(),
        contact_email: item.contact_email.clone(),
        checksum: text::truncate(&item.checksum, 64),
        name: item.name.clone(),
        ends_at_text: item.ends_at.to_string(),
        quantity_text: item.quantity.to_string(),
        amount: format!("{:.2}", item.amount),
        description: text::truncate(&item.description, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_adjustment_views(items: &[DashboardAdjustment]) -> Vec<DashboardAdjustmentView> {
    let mut views: Vec<DashboardAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_adjustment_view(item));
    }
    views
}

pub fn sum_dashboard_adjustment_view(view: &DashboardAdjustmentView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_revision as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_adjustment("code-758", "title-585");
        let view = to_dashboard_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
