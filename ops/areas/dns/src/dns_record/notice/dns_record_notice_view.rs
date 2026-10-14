use super::*;

/// How screens and exports show a dns record notice.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNoticeView {
    pub code_text: String,
    pub name: String,
    pub grace_days_text: String,
    pub ttl_seconds: String,
    pub fingerprint_text: String,
    pub revision: String,
    pub expires_at: String,
    pub margin: String,
    pub ceiling: String,
    pub score: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_dns_record_notice_view(item: &DnsRecordNotice) -> DnsRecordNoticeView {
    DnsRecordNoticeView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        grace_days_text: item.grace_days.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        revision: item.revision.to_string(),
        expires_at: item.expires_at.to_string(),
        margin: format!("{:.2}", item.margin),
        ceiling: format!("{:.2}", item.ceiling),
        score: format!("{:.2}", item.score),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_notice_views(items: &[DnsRecordNotice]) -> Vec<DnsRecordNoticeView> {
    let mut views: Vec<DnsRecordNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_notice_view(item));
    }
    views
}

pub fn sum_dns_record_notice_view(view: &DnsRecordNoticeView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_ttl_seconds as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_notice("code-880", "name-100");
        let view = to_dns_record_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
