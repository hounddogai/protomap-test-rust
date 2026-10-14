use super::*;

/// How screens and exports show a audit event cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleView {
    pub code_text: String,
    pub label_text: String,
    pub sequence: String,
    pub visible: String,
    pub title_text: String,
    pub budget_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_audit_event_cycle_view(item: &AuditEventCycle) -> AuditEventCycleView {
    AuditEventCycleView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        sequence: item.sequence.to_string(),
        visible: format!("{}", item.visible),
        title_text: item.title.clone(),
        budget_text: format!("{:.2}", item.budget),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_cycle_views(items: &[AuditEventCycle]) -> Vec<AuditEventCycleView> {
    let mut views: Vec<AuditEventCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_cycle_view(item));
    }
    views
}

pub fn sum_audit_event_cycle_view(view: &AuditEventCycleView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_cycle("code-526", "label-698");
        let view = to_audit_event_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
