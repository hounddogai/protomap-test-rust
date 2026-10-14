use super::*;

use acme_kit::text;

/// How screens and exports show a pending audit event profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventProfileView {
    pub code: String,
    pub title_text: String,
    pub updated_at: String,
    pub issuer_text: String,
    pub phone_text: String,
    pub region: String,
    pub expires_at_text: String,
    pub ttl_seconds: String,
    pub floor_value: String,
    pub name: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_pending_audit_event_profile_view(item: &PendingAuditEventProfile) -> PendingAuditEventProfileView {
    PendingAuditEventProfileView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        updated_at: item.updated_at.to_string(),
        issuer_text: text::truncate(&item.issuer, 120),
        phone_text: text::truncate(&item.phone, 40),
        region: text::truncate(&item.region, 40),
        expires_at_text: item.expires_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        name: item.name.clone(),
        currency: text::truncate(&item.currency, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_audit_event_profile_views(items: &[PendingAuditEventProfile]) -> Vec<PendingAuditEventProfileView> {
    let mut views: Vec<PendingAuditEventProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_audit_event_profile_view(item));
    }
    views
}

pub fn sum_pending_audit_event_profile_view(view: &PendingAuditEventProfileView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_expires_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_audit_event_profile("code-726", "title-915");
        let view = to_pending_audit_event_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_audit_event_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
