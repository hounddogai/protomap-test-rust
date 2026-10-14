use super::*;

use acme_kit::text;

/// How screens and exports show a dns record template.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateView {
    pub code_text: String,
    pub title: String,
    pub discount: String,
    pub host: String,
    pub contact_email: String,
    pub visible: String,
    pub priority: String,
    pub weight_text: String,
    pub phone: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_dns_record_template_view(item: &DnsRecordTemplate) -> DnsRecordTemplateView {
    DnsRecordTemplateView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        discount: format!("{:.2}", item.discount),
        host: item.host.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        visible: format!("{}", item.visible),
        priority: item.priority.to_string(),
        weight_text: format!("{:.2}", item.weight),
        phone: item.phone.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_template_views(items: &[DnsRecordTemplate]) -> Vec<DnsRecordTemplateView> {
    let mut views: Vec<DnsRecordTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_template_view(item));
    }
    views
}

pub fn sum_dns_record_template_view(view: &DnsRecordTemplateView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_discount + (parsed_priority as f64) + parsed_weight)
}
