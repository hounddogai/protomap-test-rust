use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled postmortem schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemScheduleView {
    pub code: String,
    pub title_text: String,
    pub pinned: String,
    pub updated_at: String,
    pub channel: String,
    pub threshold: String,
    pub timezone_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_scheduled_postmortem_schedule_view(item: &ScheduledPostmortemSchedule) -> ScheduledPostmortemScheduleView {
    ScheduledPostmortemScheduleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        pinned: format!("{}", item.pinned),
        updated_at: item.updated_at.to_string(),
        channel: item.channel.clone(),
        threshold: format!("{:.2}", item.threshold),
        timezone_text: text::truncate(&item.timezone, 64),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_postmortem_schedule_views(
    items: &[ScheduledPostmortemSchedule],
) -> Vec<ScheduledPostmortemScheduleView> {
    let mut views: Vec<ScheduledPostmortemScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_postmortem_schedule_view(item));
    }
    views
}

pub fn sum_scheduled_postmortem_schedule_view(view: &ScheduledPostmortemScheduleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_threshold + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_postmortem_schedule("code-396", "title-170");
        let view = to_scheduled_postmortem_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_postmortem_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
