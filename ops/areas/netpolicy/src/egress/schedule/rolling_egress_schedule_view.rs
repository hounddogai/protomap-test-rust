use super::*;

use acme_kit::text;

/// How screens and exports show a rolling egress schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressScheduleView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub contact_email_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_rolling_egress_schedule_view(item: &RollingEgressSchedule) -> RollingEgressScheduleView {
    RollingEgressScheduleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        currency: item.currency.clone(),
        contact_email_text: text::truncate(&item.contact_email, 40),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_egress_schedule_views(items: &[RollingEgressSchedule]) -> Vec<RollingEgressScheduleView> {
    let mut views: Vec<RollingEgressScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_egress_schedule_view(item));
    }
    views
}

pub fn sum_rolling_egress_schedule_view(view: &RollingEgressScheduleView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_egress_schedule("code-848", "label-931");
        let view = to_rolling_egress_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_egress_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
