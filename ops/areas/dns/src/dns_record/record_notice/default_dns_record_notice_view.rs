use super::*;

/// How screens and exports show a default dns record notice.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNoticeView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub country: String,
    pub capacity: String,
    pub attempts: String,
    pub amount: String,
    pub sequence_text: String,
    pub threshold: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_default_dns_record_notice_view(item: &DefaultDnsRecordNotice) -> DefaultDnsRecordNoticeView {
    DefaultDnsRecordNoticeView {
        code: item.code.clone(),
        title: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        country: item.country.clone(),
        capacity: item.capacity.to_string(),
        attempts: item.attempts.to_string(),
        amount: format!("{:.2}", item.amount),
        sequence_text: item.sequence.to_string(),
        threshold: format!("{:.2}", item.threshold),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_dns_record_notice_views(items: &[DefaultDnsRecordNotice]) -> Vec<DefaultDnsRecordNoticeView> {
    let mut views: Vec<DefaultDnsRecordNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_dns_record_notice_view(item));
    }
    views
}

pub fn sum_default_dns_record_notice_view(view: &DefaultDnsRecordNoticeView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ratio + (parsed_capacity as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_dns_record_notice("code-183", "title-396");
        let view = to_default_dns_record_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_dns_record_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
