use super::*;

use acme_kit::text;

/// How screens and exports show a lenient audit event group.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventGroupView {
    pub code: String,
    pub title: String,
    pub grace_days: String,
    pub timezone: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_lenient_audit_event_group_view(item: &LenientAuditEventGroup) -> LenientAuditEventGroupView {
    LenientAuditEventGroupView {
        code: item.code.clone(),
        title: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        timezone: text::truncate(&item.timezone, 80),
        region_text: text::truncate(&item.region, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_audit_event_group_views(items: &[LenientAuditEventGroup]) -> Vec<LenientAuditEventGroupView> {
    let mut views: Vec<LenientAuditEventGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_audit_event_group_view(item));
    }
    views
}

pub fn sum_lenient_audit_event_group_view(view: &LenientAuditEventGroupView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_audit_event_group("code-915", "title-844");
        let view = to_lenient_audit_event_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_audit_event_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
