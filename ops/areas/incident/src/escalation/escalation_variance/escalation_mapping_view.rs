use super::*;

use acme_kit::text;

/// How screens and exports show a escalation mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMappingView {
    pub code: String,
    pub name: String,
    pub archived_text: String,
    pub currency: String,
    pub priority: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_escalation_mapping_view(item: &EscalationMapping) -> EscalationMappingView {
    EscalationMappingView {
        code: item.code.clone(),
        name: item.name.clone(),
        archived_text: format!("{}", item.archived),
        currency: text::truncate(&item.currency, 120),
        priority: item.priority.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_mapping_views(items: &[EscalationMapping]) -> Vec<EscalationMappingView> {
    let mut views: Vec<EscalationMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_mapping_view(item));
    }
    views
}

pub fn sum_escalation_mapping_view(view: &EscalationMappingView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_mapping("code-70", "name-448");
        let view = to_escalation_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
