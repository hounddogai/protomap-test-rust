use super::*;

use acme_kit::text;

/// How screens and exports show a grant audit.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAuditView {
    pub code: String,
    pub title_text: String,
    pub quantity: String,
    pub region_text: String,
    pub ttl_seconds_text: String,
    pub retries_text: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_grant_audit_view(item: &GrantAudit) -> GrantAuditView {
    GrantAuditView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        quantity: item.quantity.to_string(),
        region_text: item.region.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        retries_text: item.retries.to_string(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_audit_views(items: &[GrantAudit]) -> Vec<GrantAuditView> {
    let mut views: Vec<GrantAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_audit_view(item));
    }
    views
}

pub fn sum_grant_audit_view(view: &GrantAuditView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_ttl_seconds as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_audit("code-268", "title-653");
        let view = to_grant_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
