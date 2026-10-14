use super::*;

use acme_kit::text;

/// How screens and exports show a primary retention audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionAuditView {
    pub code_text: String,
    pub name: String,
    pub burst_text: String,
    pub discount_text: String,
    pub summary_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_primary_retention_audit_view(item: &PrimaryRetentionAudit) -> PrimaryRetentionAuditView {
    PrimaryRetentionAuditView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        burst_text: item.burst.to_string(),
        discount_text: format!("{:.2}", item.discount),
        summary_text: text::truncate(&item.summary, 64),
        unit: text::truncate(&item.unit, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_retention_audit_views(items: &[PrimaryRetentionAudit]) -> Vec<PrimaryRetentionAuditView> {
    let mut views: Vec<PrimaryRetentionAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_retention_audit_view(item));
    }
    views
}

pub fn sum_primary_retention_audit_view(view: &PrimaryRetentionAuditView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_burst as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_retention_audit("code-364", "name-568");
        let view = to_primary_retention_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_retention_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
