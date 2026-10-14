use super::*;

use acme_kit::text;

/// How screens and exports show a dns record window.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordWindowView {
    pub code_text: String,
    pub title_text: String,
    pub pinned: String,
    pub approved_text: String,
    pub quota_units: String,
    pub capacity_text: String,
    pub currency_text: String,
    pub reviewer: String,
    pub active: String,
    pub source_text: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_dns_record_window_view(item: &DnsRecordWindow) -> DnsRecordWindowView {
    DnsRecordWindowView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        pinned: format!("{}", item.pinned),
        approved_text: format!("{}", item.approved),
        quota_units: item.quota_units.to_string(),
        capacity_text: item.capacity.to_string(),
        currency_text: text::truncate(&item.currency, 64),
        reviewer: text::truncate(&item.reviewer, 120),
        active: format!("{}", item.active),
        source_text: text::truncate(&item.source, 80),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_window_views(items: &[DnsRecordWindow]) -> Vec<DnsRecordWindowView> {
    let mut views: Vec<DnsRecordWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_window_view(item));
    }
    views
}

pub fn sum_dns_record_window_view(view: &DnsRecordWindowView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_capacity as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_window("code-961", "title-324");
        let view = to_dns_record_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
