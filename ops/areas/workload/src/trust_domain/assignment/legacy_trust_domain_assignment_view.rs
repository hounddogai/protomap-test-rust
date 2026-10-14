use super::*;

/// How screens and exports show a legacy trust domain assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignmentView {
    pub code: String,
    pub title_text: String,
    pub port: String,
    pub window_days: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_legacy_trust_domain_assignment_view(item: &LegacyTrustDomainAssignment) -> LegacyTrustDomainAssignmentView {
    LegacyTrustDomainAssignmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        port: item.port.to_string(),
        window_days: item.window_days.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_trust_domain_assignment_views(
    items: &[LegacyTrustDomainAssignment],
) -> Vec<LegacyTrustDomainAssignmentView> {
    let mut views: Vec<LegacyTrustDomainAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_trust_domain_assignment_view(item));
    }
    views
}

pub fn sum_legacy_trust_domain_assignment_view(view: &LegacyTrustDomainAssignmentView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_port as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_trust_domain_assignment("code-375", "title-910");
        let view = to_legacy_trust_domain_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_trust_domain_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
