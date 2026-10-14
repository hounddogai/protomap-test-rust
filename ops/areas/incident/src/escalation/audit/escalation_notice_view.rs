use super::*;

/// How screens and exports show a escalation notice.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeView {
    pub code: String,
    pub label_text: String,
    pub floor_value: String,
    pub notes: String,
    pub active: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_escalation_notice_view(item: &EscalationNotice) -> EscalationNoticeView {
    EscalationNoticeView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        notes: item.notes.clone(),
        active: format!("{}", item.active),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_notice_views(items: &[EscalationNotice]) -> Vec<EscalationNoticeView> {
    let mut views: Vec<EscalationNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_notice_view(item));
    }
    views
}

pub fn sum_escalation_notice_view(view: &EscalationNoticeView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_floor_value + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_notice("code-418", "label-165");
        let view = to_escalation_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
