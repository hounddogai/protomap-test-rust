use super::*;

use acme_kit::text;

/// How screens and exports show a escalation cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCycleView {
    pub code: String,
    pub name: String,
    pub approved: String,
    pub created_at_text: String,
    pub scope: String,
    pub principal_text: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_escalation_cycle_view(item: &EscalationCycle) -> EscalationCycleView {
    EscalationCycleView {
        code: item.code.clone(),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        created_at_text: item.created_at.to_string(),
        scope: item.scope.clone(),
        principal_text: text::truncate(&item.principal, 40),
        label: text::truncate(&item.label, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_cycle_views(items: &[EscalationCycle]) -> Vec<EscalationCycleView> {
    let mut views: Vec<EscalationCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_cycle_view(item));
    }
    views
}

pub fn sum_escalation_cycle_view(view: &EscalationCycleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_cycle("code-919", "name-270");
        let view = to_escalation_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
