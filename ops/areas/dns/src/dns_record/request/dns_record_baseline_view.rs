use super::*;

use acme_kit::text;

/// How screens and exports show a dns record baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaselineView {
    pub code_text: String,
    pub title: String,
    pub limit: String,
    pub threshold: String,
    pub issuer: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_dns_record_baseline_view(item: &DnsRecordBaseline) -> DnsRecordBaselineView {
    DnsRecordBaselineView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        limit: item.limit.to_string(),
        threshold: format!("{:.2}", item.threshold),
        issuer: text::truncate(&item.issuer, 80),
        label: text::truncate(&item.label, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_baseline_views(items: &[DnsRecordBaseline]) -> Vec<DnsRecordBaselineView> {
    let mut views: Vec<DnsRecordBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_baseline_view(item));
    }
    views
}

pub fn sum_dns_record_baseline_view(view: &DnsRecordBaselineView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_limit as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_baseline("code-979", "title-269");
        let view = to_dns_record_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
