use super::*;

use acme_kit::text;

/// How screens and exports show a default dns record segment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordSegmentView {
    pub code_text: String,
    pub name_text: String,
    pub expires_at: String,
    pub active: String,
    pub fingerprint: String,
    pub floor_value: String,
    pub amount: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_default_dns_record_segment_view(item: &DefaultDnsRecordSegment) -> DefaultDnsRecordSegmentView {
    DefaultDnsRecordSegmentView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        expires_at: item.expires_at.to_string(),
        active: format!("{}", item.active),
        fingerprint: item.fingerprint.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        amount: format!("{:.2}", item.amount),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_dns_record_segment_views(items: &[DefaultDnsRecordSegment]) -> Vec<DefaultDnsRecordSegmentView> {
    let mut views: Vec<DefaultDnsRecordSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_dns_record_segment_view(item));
    }
    views
}

pub fn sum_default_dns_record_segment_view(view: &DefaultDnsRecordSegmentView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_floor_value + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_dns_record_segment("code-209", "name-901");
        let view = to_default_dns_record_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_dns_record_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
