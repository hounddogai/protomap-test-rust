use super::*;

use acme_kit::text;

/// How screens and exports show a page audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAuditView {
    pub code: String,
    pub label_text: String,
    pub issuer: String,
    pub contact_email: String,
    pub category: String,
    pub phone: String,
    pub approved: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_page_audit_view(item: &PageAudit) -> PageAuditView {
    PageAuditView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        issuer: text::truncate(&item.issuer, 80),
        contact_email: text::truncate(&item.contact_email, 64),
        category: text::truncate(&item.category, 64),
        phone: item.phone.clone(),
        approved: format!("{}", item.approved),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_audit_views(items: &[PageAudit]) -> Vec<PageAuditView> {
    let mut views: Vec<PageAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_audit_view(item));
    }
    views
}

pub fn sum_page_audit_view(view: &PageAuditView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_audit("code-13", "label-297");
        let view = to_page_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
