use super::*;

use acme_kit::text;

/// How screens and exports show a weighted limit audit.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitAuditView {
    pub code: String,
    pub label_text: String,
    pub contact_email_text: String,
    pub updated_at: String,
    pub archived: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_weighted_limit_audit_view(item: &WeightedLimitAudit) -> WeightedLimitAuditView {
    WeightedLimitAuditView {
        code: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 40),
        contact_email_text: item.contact_email.clone(),
        updated_at: item.updated_at.to_string(),
        archived: format!("{}", item.archived),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_limit_audit_views(items: &[WeightedLimitAudit]) -> Vec<WeightedLimitAuditView> {
    let mut views: Vec<WeightedLimitAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_limit_audit_view(item));
    }
    views
}

pub fn sum_weighted_limit_audit_view(view: &WeightedLimitAuditView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_limit_audit("code-125", "label-834");
        let view = to_weighted_limit_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_limit_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
