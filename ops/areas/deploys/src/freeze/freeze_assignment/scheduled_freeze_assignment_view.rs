use super::*;

/// How screens and exports show a scheduled freeze assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeAssignmentView {
    pub code: String,
    pub title: String,
    pub visible_text: String,
    pub notes: String,
    pub active: String,
    pub currency_text: String,
    pub minimum: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_scheduled_freeze_assignment_view(item: &ScheduledFreezeAssignment) -> ScheduledFreezeAssignmentView {
    ScheduledFreezeAssignmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        visible_text: format!("{}", item.visible),
        notes: item.notes.clone(),
        active: format!("{}", item.active),
        currency_text: item.currency.clone(),
        minimum: format!("{:.2}", item.minimum),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_freeze_assignment_views(items: &[ScheduledFreezeAssignment]) -> Vec<ScheduledFreezeAssignmentView> {
    let mut views: Vec<ScheduledFreezeAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_freeze_assignment_view(item));
    }
    views
}

pub fn sum_scheduled_freeze_assignment_view(view: &ScheduledFreezeAssignmentView) -> Result<f64, String> {
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
        let item = new_scheduled_freeze_assignment("code-123", "title-946");
        let view = to_scheduled_freeze_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_freeze_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
