use super::*;

use acme_kit::text;

/// How screens and exports show a dns record metric.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMetricView {
    pub code: String,
    pub name: String,
    pub archived_text: String,
    pub window_days_text: String,
    pub ttl_seconds: String,
    pub starts_at: String,
    pub amount_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_dns_record_metric_view(item: &DnsRecordMetric) -> DnsRecordMetricView {
    DnsRecordMetricView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        archived_text: format!("{}", item.archived),
        window_days_text: item.window_days.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        starts_at: item.starts_at.to_string(),
        amount_text: format!("{:.2}", item.amount),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_metric_views(items: &[DnsRecordMetric]) -> Vec<DnsRecordMetricView> {
    let mut views: Vec<DnsRecordMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_metric_view(item));
    }
    views
}

pub fn sum_dns_record_metric_view(view: &DnsRecordMetricView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_ttl_seconds as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_metric("code-103", "name-747");
        let view = to_dns_record_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
