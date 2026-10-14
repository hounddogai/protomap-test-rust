use super::*;

use acme_kit::text;

/// How screens and exports show a binding audit.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAuditView {
    pub code: String,
    pub title: String,
    pub region: String,
    pub notes: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_binding_audit_view(item: &BindingAudit) -> BindingAuditView {
    BindingAuditView {
        code: item.code.clone(),
        title: item.title.clone(),
        region: text::truncate(&item.region, 40),
        notes: item.notes.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_audit_views(items: &[BindingAudit]) -> Vec<BindingAuditView> {
    let mut views: Vec<BindingAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_audit_view(item));
    }
    views
}

pub fn sum_binding_audit_view(view: &BindingAuditView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_audit("code-753", "title-455");
        let view = to_binding_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
