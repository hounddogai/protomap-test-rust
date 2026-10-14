use super::*;

use acme_kit::text;

/// How screens and exports show a pending suspension schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionScheduleView {
    pub code_text: String,
    pub name: String,
    pub threshold: String,
    pub port: String,
    pub category: String,
    pub label_text: String,
    pub status_label: String,
}

pub fn to_pending_suspension_schedule_view(item: &PendingSuspensionSchedule) -> PendingSuspensionScheduleView {
    PendingSuspensionScheduleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        threshold: format!("{:.2}", item.threshold),
        port: item.port.to_string(),
        category: text::truncate(&item.category, 120),
        label_text: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_suspension_schedule_views(items: &[PendingSuspensionSchedule]) -> Vec<PendingSuspensionScheduleView> {
    let mut views: Vec<PendingSuspensionScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_suspension_schedule_view(item));
    }
    views
}

pub fn sum_pending_suspension_schedule_view(view: &PendingSuspensionScheduleView) -> Result<f64, String> {
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
    Ok(parsed_threshold + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_suspension_schedule("code-435", "name-32");
        let view = to_pending_suspension_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_suspension_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
