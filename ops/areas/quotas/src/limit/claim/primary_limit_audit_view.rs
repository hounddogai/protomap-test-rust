use super::*;

use acme_kit::text;

/// How screens and exports show a primary limit audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAuditView {
    pub code: String,
    pub title: String,
    pub country_text: String,
    pub notes: String,
    pub cost_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_primary_limit_audit_view(item: &PrimaryLimitAudit) -> PrimaryLimitAuditView {
    PrimaryLimitAuditView {
        code: item.code.clone(),
        title: item.title.clone(),
        country_text: item.country.clone(),
        notes: item.notes.clone(),
        cost_text: format!("{:.2}", item.cost),
        cluster: text::truncate(&item.cluster, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_limit_audit_views(items: &[PrimaryLimitAudit]) -> Vec<PrimaryLimitAuditView> {
    let mut views: Vec<PrimaryLimitAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_limit_audit_view(item));
    }
    views
}

pub fn sum_primary_limit_audit_view(view: &PrimaryLimitAuditView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_limit_audit("code-816", "title-424");
        let view = to_primary_limit_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_limit_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
