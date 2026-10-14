use super::*;

use acme_kit::text;

/// How screens and exports show a dns record override.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordOverrideView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub owner_id: String,
    pub revision: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_dns_record_override_view(item: &DnsRecordOverride) -> DnsRecordOverrideView {
    DnsRecordOverrideView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        cipher: item.cipher.clone(),
        owner_id: item.owner_id.clone(),
        revision: item.revision.to_string(),
        label: text::truncate(&item.label, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_override_views(items: &[DnsRecordOverride]) -> Vec<DnsRecordOverrideView> {
    let mut views: Vec<DnsRecordOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_override_view(item));
    }
    views
}

pub fn sum_dns_record_override_view(view: &DnsRecordOverrideView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_override("code-916", "title-29");
        let view = to_dns_record_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
