use super::*;

/// How screens and exports show a timeline audit.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAuditView {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub revision: String,
    pub contact_email: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_timeline_audit_view(item: &TimelineAudit) -> TimelineAuditView {
    TimelineAuditView {
        code: item.code.clone(),
        name: item.name.clone(),
        external_ref: item.external_ref.clone(),
        revision: item.revision.to_string(),
        contact_email: item.contact_email.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_audit_views(items: &[TimelineAudit]) -> Vec<TimelineAuditView> {
    let mut views: Vec<TimelineAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_audit_view(item));
    }
    views
}

pub fn sum_timeline_audit_view(view: &TimelineAuditView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_audit("code-989", "name-790");
        let view = to_timeline_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
