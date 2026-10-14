use super::*;

/// How screens and exports show a strict retention schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionScheduleView {
    pub code: String,
    pub label: String,
    pub priority: String,
    pub margin_text: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_strict_retention_schedule_view(item: &StrictRetentionSchedule) -> StrictRetentionScheduleView {
    StrictRetentionScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        margin_text: format!("{:.2}", item.margin),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_retention_schedule_views(items: &[StrictRetentionSchedule]) -> Vec<StrictRetentionScheduleView> {
    let mut views: Vec<StrictRetentionScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_retention_schedule_view(item));
    }
    views
}

pub fn sum_strict_retention_schedule_view(view: &StrictRetentionScheduleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_priority as f64 + parsed_margin + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_retention_schedule("code-635", "label-324");
        let view = to_strict_retention_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_retention_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
