use super::*;

use acme_kit::text;

/// How screens and exports show a dns record variance.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordVarianceView {
    pub code_text: String,
    pub title_text: String,
    pub amount: String,
    pub ratio: String,
    pub summary_text: String,
    pub label: String,
    pub cipher: String,
    pub visible_text: String,
    pub cost: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_dns_record_variance_view(item: &DnsRecordVariance) -> DnsRecordVarianceView {
    DnsRecordVarianceView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        amount: format!("{:.2}", item.amount),
        ratio: format!("{:.2}", item.ratio),
        summary_text: item.summary.clone(),
        label: item.label.clone(),
        cipher: item.cipher.clone(),
        visible_text: format!("{}", item.visible),
        cost: format!("{:.2}", item.cost),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_variance_views(items: &[DnsRecordVariance]) -> Vec<DnsRecordVarianceView> {
    let mut views: Vec<DnsRecordVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_variance_view(item));
    }
    views
}

pub fn sum_dns_record_variance_view(view: &DnsRecordVarianceView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_amount + parsed_ratio + parsed_cost)
}
