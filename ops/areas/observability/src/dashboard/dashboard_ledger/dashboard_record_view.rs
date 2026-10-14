use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard record.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRecordView {
    pub code: String,
    pub name: String,
    pub floor_value: String,
    pub maximum: String,
    pub quantity: String,
    pub grace_days: String,
    pub expires_at: String,
    pub scope_text: String,
    pub ends_at: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_dashboard_record_view(item: &DashboardRecord) -> DashboardRecordView {
    DashboardRecordView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        floor_value: format!("{:.2}", item.floor_value),
        maximum: format!("{:.2}", item.maximum),
        quantity: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        expires_at: item.expires_at.to_string(),
        scope_text: text::truncate(&item.scope, 120),
        ends_at: item.ends_at.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_record_views(items: &[DashboardRecord]) -> Vec<DashboardRecordView> {
    let mut views: Vec<DashboardRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_record_view(item));
    }
    views
}

pub fn sum_dashboard_record_view(view: &DashboardRecordView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_floor_value + parsed_maximum + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_record("code-236", "name-184");
        let view = to_dashboard_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
