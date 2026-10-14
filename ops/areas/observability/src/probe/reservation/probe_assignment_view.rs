use super::*;

use acme_kit::text;

/// How screens and exports show a probe assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAssignmentView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub window_days_text: String,
    pub country: String,
    pub unit: String,
    pub limit_text: String,
    pub expires_at: String,
    pub margin: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_probe_assignment_view(item: &ProbeAssignment) -> ProbeAssignmentView {
    ProbeAssignmentView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 40),
        ends_at: item.ends_at.to_string(),
        window_days_text: item.window_days.to_string(),
        country: text::truncate(&item.country, 80),
        unit: item.unit.clone(),
        limit_text: item.limit.to_string(),
        expires_at: item.expires_at.to_string(),
        margin: format!("{:.2}", item.margin),
        locale: text::truncate(&item.locale, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_assignment_views(items: &[ProbeAssignment]) -> Vec<ProbeAssignmentView> {
    let mut views: Vec<ProbeAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_assignment_view(item));
    }
    views
}

pub fn sum_probe_assignment_view(view: &ProbeAssignmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_window_days as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_assignment("code-224", "name-765");
        let view = to_probe_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
