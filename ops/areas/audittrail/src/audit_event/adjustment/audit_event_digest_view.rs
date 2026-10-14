use super::*;

use acme_kit::text;

/// How screens and exports show a audit event digest.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDigestView {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub external_ref: String,
    pub burst: String,
    pub ends_at: String,
    pub country: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_audit_event_digest_view(item: &AuditEventDigest) -> AuditEventDigestView {
    AuditEventDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        scope: item.scope.clone(),
        external_ref: item.external_ref.clone(),
        burst: item.burst.to_string(),
        ends_at: item.ends_at.to_string(),
        country: text::truncate(&item.country, 40),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_digest_views(items: &[AuditEventDigest]) -> Vec<AuditEventDigestView> {
    let mut views: Vec<AuditEventDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_digest_view(item));
    }
    views
}

pub fn sum_audit_event_digest_view(view: &AuditEventDigestView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_digest("code-848", "title-417");
        let view = to_audit_event_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
