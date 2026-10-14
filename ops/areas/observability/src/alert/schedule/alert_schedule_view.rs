use super::*;

use acme_kit::text;

/// How screens and exports show a alert schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertScheduleView {
    pub code: String,
    pub label_text: String,
    pub description: String,
    pub external_ref: String,
    pub rate: String,
    pub discount: String,
    pub title_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_alert_schedule_view(item: &AlertSchedule) -> AlertScheduleView {
    AlertScheduleView {
        code: text::truncate(&item.code, 80),
        label_text: text::truncate(&item.label, 64),
        description: text::truncate(&item.description, 80),
        external_ref: item.external_ref.clone(),
        rate: format!("{:.2}", item.rate),
        discount: format!("{:.2}", item.discount),
        title_text: text::truncate(&item.title, 64),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_schedule_views(items: &[AlertSchedule]) -> Vec<AlertScheduleView> {
    let mut views: Vec<AlertScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_schedule_view(item));
    }
    views
}

pub fn sum_alert_schedule_view(view: &AlertScheduleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_rate + parsed_discount + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_schedule("code-131", "label-123");
        let view = to_alert_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
