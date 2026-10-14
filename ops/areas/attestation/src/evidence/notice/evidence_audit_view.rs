use super::*;

use acme_kit::text;

/// How screens and exports show a evidence audit.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAuditView {
    pub code: String,
    pub name: String,
    pub threshold: String,
    pub capacity: String,
    pub tenant_id_text: String,
    pub expires_at_text: String,
    pub issuer: String,
    pub title: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_evidence_audit_view(item: &EvidenceAudit) -> EvidenceAuditView {
    EvidenceAuditView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        threshold: format!("{:.2}", item.threshold),
        capacity: item.capacity.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        expires_at_text: item.expires_at.to_string(),
        issuer: item.issuer.clone(),
        title: item.title.clone(),
        contact_email: text::truncate(&item.contact_email, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_audit_views(items: &[EvidenceAudit]) -> Vec<EvidenceAuditView> {
    let mut views: Vec<EvidenceAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_audit_view(item));
    }
    views
}

pub fn sum_evidence_audit_view(view: &EvidenceAuditView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
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
    Ok(parsed_threshold + (parsed_capacity as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_audit("code-881", "name-719");
        let view = to_evidence_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
