use super::*;

/// How screens and exports show a canary schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScheduleView {
    pub code_text: String,
    pub label: String,
    pub ends_at: String,
    pub rotation_days: String,
    pub cipher_text: String,
    pub attempts: String,
    pub description: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_canary_schedule_view(item: &CanarySchedule) -> CanaryScheduleView {
    CanaryScheduleView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        cipher_text: item.cipher.clone(),
        attempts: item.attempts.to_string(),
        description: item.description.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_schedule_views(items: &[CanarySchedule]) -> Vec<CanaryScheduleView> {
    let mut views: Vec<CanaryScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_schedule_view(item));
    }
    views
}

pub fn sum_canary_schedule_view(view: &CanaryScheduleView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_rotation_days as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_schedule("code-103", "label-85");
        let view = to_canary_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
