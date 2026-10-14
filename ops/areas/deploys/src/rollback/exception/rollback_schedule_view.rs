use super::*;

use acme_kit::text;

/// How screens and exports show a rollback schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleView {
    pub code: String,
    pub label: String,
    pub notes_text: String,
    pub minimum: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_rollback_schedule_view(item: &RollbackSchedule) -> RollbackScheduleView {
    RollbackScheduleView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        notes_text: text::truncate(&item.notes, 80),
        minimum: format!("{:.2}", item.minimum),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_schedule_views(items: &[RollbackSchedule]) -> Vec<RollbackScheduleView> {
    let mut views: Vec<RollbackScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_schedule_view(item));
    }
    views
}

pub fn sum_rollback_schedule_view(view: &RollbackScheduleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_schedule("code-441", "label-844");
        let view = to_rollback_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
