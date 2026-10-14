use super::*;

use acme_kit::text;

/// How screens and exports show a usage audit.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAuditView {
    pub code_text: String,
    pub label: String,
    pub currency_text: String,
    pub minimum: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_usage_audit_view(item: &UsageAudit) -> UsageAuditView {
    UsageAuditView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        currency_text: text::truncate(&item.currency, 40),
        minimum: format!("{:.2}", item.minimum),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_audit_views(items: &[UsageAudit]) -> Vec<UsageAuditView> {
    let mut views: Vec<UsageAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_audit_view(item));
    }
    views
}

pub fn sum_usage_audit_view(view: &UsageAuditView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_minimum + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_audit("code-727", "label-699");
        let view = to_usage_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
