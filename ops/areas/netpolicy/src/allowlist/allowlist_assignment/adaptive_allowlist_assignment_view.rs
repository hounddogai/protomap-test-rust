use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive allowlist assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistAssignmentView {
    pub code: String,
    pub name_text: String,
    pub duration_minutes_text: String,
    pub amount: String,
    pub external_ref: String,
    pub margin_text: String,
    pub port_text: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_adaptive_allowlist_assignment_view(item: &AdaptiveAllowlistAssignment) -> AdaptiveAllowlistAssignmentView {
    AdaptiveAllowlistAssignmentView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        amount: format!("{:.2}", item.amount),
        external_ref: text::truncate(&item.external_ref, 40),
        margin_text: format!("{:.2}", item.margin),
        port_text: item.port.to_string(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_allowlist_assignment_views(
    items: &[AdaptiveAllowlistAssignment],
) -> Vec<AdaptiveAllowlistAssignmentView> {
    let mut views: Vec<AdaptiveAllowlistAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_allowlist_assignment_view(item));
    }
    views
}

pub fn sum_adaptive_allowlist_assignment_view(view: &AdaptiveAllowlistAssignmentView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_amount + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_allowlist_assignment("code-959", "name-679");
        let view = to_adaptive_allowlist_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_allowlist_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
