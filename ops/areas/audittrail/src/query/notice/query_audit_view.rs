use super::*;

use acme_kit::text;

/// How screens and exports show a query audit.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAuditView {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub quantity: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_query_audit_view(item: &QueryAudit) -> QueryAuditView {
    QueryAuditView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 64),
        issuer: item.issuer.clone(),
        quantity: item.quantity.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_audit_views(items: &[QueryAudit]) -> Vec<QueryAuditView> {
    let mut views: Vec<QueryAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_audit_view(item));
    }
    views
}

pub fn sum_query_audit_view(view: &QueryAuditView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_audit("code-583", "title-166");
        let view = to_query_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
