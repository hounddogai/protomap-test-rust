use super::*;

use acme_kit::text;

/// How screens and exports show a tiered peering audit.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub external_ref: String,
    pub ratio: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_tiered_peering_audit_view(item: &TieredPeeringAudit) -> TieredPeeringAuditView {
    TieredPeeringAuditView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        external_ref: text::truncate(&item.external_ref, 64),
        ratio: format!("{:.2}", item.ratio),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_peering_audit_views(items: &[TieredPeeringAudit]) -> Vec<TieredPeeringAuditView> {
    let mut views: Vec<TieredPeeringAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_peering_audit_view(item));
    }
    views
}

pub fn sum_tiered_peering_audit_view(view: &TieredPeeringAuditView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_revision as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_peering_audit("code-688", "name-230");
        let view = to_tiered_peering_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_peering_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
