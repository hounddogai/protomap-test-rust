use super::*;

use acme_kit::text;

/// How screens and exports show a weighted alert schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertScheduleView {
    pub code: String,
    pub name_text: String,
    pub checksum: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_weighted_alert_schedule_view(item: &WeightedAlertSchedule) -> WeightedAlertScheduleView {
    WeightedAlertScheduleView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        checksum: item.checksum.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_alert_schedule_views(items: &[WeightedAlertSchedule]) -> Vec<WeightedAlertScheduleView> {
    let mut views: Vec<WeightedAlertScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_alert_schedule_view(item));
    }
    views
}

pub fn sum_weighted_alert_schedule_view(view: &WeightedAlertScheduleView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_alert_schedule("code-579", "name-92");
        let view = to_weighted_alert_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_alert_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
