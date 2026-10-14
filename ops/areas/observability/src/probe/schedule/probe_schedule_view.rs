use super::*;

use acme_kit::text;

/// How screens and exports show a probe schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScheduleView {
    pub code_text: String,
    pub label_text: String,
    pub grace_days: String,
    pub attempts: String,
    pub title: String,
    pub limit: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_probe_schedule_view(item: &ProbeSchedule) -> ProbeScheduleView {
    ProbeScheduleView {
        code_text: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        attempts: item.attempts.to_string(),
        title: item.title.clone(),
        limit: item.limit.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_schedule_views(items: &[ProbeSchedule]) -> Vec<ProbeScheduleView> {
    let mut views: Vec<ProbeScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_schedule_view(item));
    }
    views
}

pub fn sum_probe_schedule_view(view: &ProbeScheduleView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_attempts as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_schedule("code-872", "label-40");
        let view = to_probe_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
