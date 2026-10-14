use super::*;

use acme_kit::text;

/// How screens and exports show a rolling federation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleView {
    pub code: String,
    pub name_text: String,
    pub quantity: String,
    pub visible_text: String,
    pub revision: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_rolling_federation_schedule_view(item: &RollingFederationSchedule) -> RollingFederationScheduleView {
    RollingFederationScheduleView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        quantity: item.quantity.to_string(),
        visible_text: format!("{}", item.visible),
        revision: item.revision.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_federation_schedule_views(items: &[RollingFederationSchedule]) -> Vec<RollingFederationScheduleView> {
    let mut views: Vec<RollingFederationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_federation_schedule_view(item));
    }
    views
}

pub fn sum_rolling_federation_schedule_view(view: &RollingFederationScheduleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_revision as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_federation_schedule("code-858", "name-457");
        let view = to_rolling_federation_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_federation_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
