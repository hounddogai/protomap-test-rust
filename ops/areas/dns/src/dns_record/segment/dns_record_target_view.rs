use super::*;

use acme_kit::text;

/// How screens and exports show a dns record target.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTargetView {
    pub code_text: String,
    pub name: String,
    pub updated_at: String,
    pub grace_days_text: String,
    pub quota_units: String,
    pub sequence: String,
    pub created_at: String,
    pub margin: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_dns_record_target_view(item: &DnsRecordTarget) -> DnsRecordTargetView {
    DnsRecordTargetView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        quota_units: item.quota_units.to_string(),
        sequence: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        margin: format!("{:.2}", item.margin),
        reviewer: text::truncate(&item.reviewer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_target_views(items: &[DnsRecordTarget]) -> Vec<DnsRecordTargetView> {
    let mut views: Vec<DnsRecordTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_target_view(item));
    }
    views
}

pub fn sum_dns_record_target_view(view: &DnsRecordTargetView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_grace_days as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_target("code-640", "name-812");
        let view = to_dns_record_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
