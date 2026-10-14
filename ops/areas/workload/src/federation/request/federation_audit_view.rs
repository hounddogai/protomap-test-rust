use super::*;

use acme_kit::text;

/// How screens and exports show a federation audit.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAuditView {
    pub code: String,
    pub title: String,
    pub ends_at: String,
    pub ttl_seconds: String,
    pub discount: String,
    pub name: String,
    pub score: String,
    pub tenant_id_text: String,
    pub approved_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_federation_audit_view(item: &FederationAudit) -> FederationAuditView {
    FederationAuditView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 64),
        ends_at: item.ends_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        discount: format!("{:.2}", item.discount),
        name: item.name.clone(),
        score: format!("{:.2}", item.score),
        tenant_id_text: text::truncate(&item.tenant_id, 64),
        approved_text: format!("{}", item.approved),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_audit_views(items: &[FederationAudit]) -> Vec<FederationAuditView> {
    let mut views: Vec<FederationAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_audit_view(item));
    }
    views
}

pub fn sum_federation_audit_view(view: &FederationAuditView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_ttl_seconds as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_audit("code-841", "title-401");
        let view = to_federation_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
