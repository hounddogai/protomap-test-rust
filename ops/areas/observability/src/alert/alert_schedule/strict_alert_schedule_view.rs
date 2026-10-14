use super::*;

use acme_kit::text;

/// How screens and exports show a strict alert schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertScheduleView {
    pub code_text: String,
    pub title_text: String,
    pub external_ref: String,
    pub phone: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_strict_alert_schedule_view(item: &StrictAlertSchedule) -> StrictAlertScheduleView {
    StrictAlertScheduleView {
        code_text: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        external_ref: item.external_ref.clone(),
        phone: item.phone.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_alert_schedule_views(items: &[StrictAlertSchedule]) -> Vec<StrictAlertScheduleView> {
    let mut views: Vec<StrictAlertScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_alert_schedule_view(item));
    }
    views
}

pub fn sum_strict_alert_schedule_view(view: &StrictAlertScheduleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_alert_schedule("code-292", "title-954");
        let view = to_strict_alert_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_alert_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
