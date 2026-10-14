use super::*;

use acme_kit::text;

/// How screens and exports show a dns record audit.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAuditView {
    pub code: String,
    pub label: String,
    pub starts_at: String,
    pub limit: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_dns_record_audit_view(item: &DnsRecordAudit) -> DnsRecordAuditView {
    DnsRecordAuditView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        starts_at: item.starts_at.to_string(),
        limit: item.limit.to_string(),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_audit_views(items: &[DnsRecordAudit]) -> Vec<DnsRecordAuditView> {
    let mut views: Vec<DnsRecordAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_audit_view(item));
    }
    views
}

pub fn sum_dns_record_audit_view(view: &DnsRecordAuditView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_audit("code-737", "label-363");
        let view = to_dns_record_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
