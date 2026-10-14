use super::*;

use acme_kit::text;

/// How screens and exports show a audit event exception.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventExceptionView {
    pub code_text: String,
    pub title: String,
    pub capacity: String,
    pub expires_at_text: String,
    pub name: String,
    pub ratio: String,
    pub grace_days: String,
    pub cipher: String,
    pub port: String,
    pub cost: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_audit_event_exception_view(item: &AuditEventException) -> AuditEventExceptionView {
    AuditEventExceptionView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        capacity: item.capacity.to_string(),
        expires_at_text: item.expires_at.to_string(),
        name: text::truncate(&item.name, 40),
        ratio: format!("{:.2}", item.ratio),
        grace_days: item.grace_days.to_string(),
        cipher: item.cipher.clone(),
        port: item.port.to_string(),
        cost: format!("{:.2}", item.cost),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_exception_views(items: &[AuditEventException]) -> Vec<AuditEventExceptionView> {
    let mut views: Vec<AuditEventExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_exception_view(item));
    }
    views
}

pub fn sum_audit_event_exception_view(view: &AuditEventExceptionView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_expires_at as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_exception("code-532", "title-366");
        let view = to_audit_event_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
