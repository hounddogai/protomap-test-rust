use super::*;

use acme_kit::text;

/// How screens and exports show a dns record plan.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPlanView {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub description_text: String,
    pub audience_text: String,
    pub cost_text: String,
    pub updated_at: String,
    pub name: String,
    pub checksum: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_dns_record_plan_view(item: &DnsRecordPlan) -> DnsRecordPlanView {
    DnsRecordPlanView {
        code: item.code.clone(),
        title: item.title.clone(),
        channel: text::truncate(&item.channel, 64),
        description_text: item.description.clone(),
        audience_text: item.audience.clone(),
        cost_text: format!("{:.2}", item.cost),
        updated_at: item.updated_at.to_string(),
        name: item.name.clone(),
        checksum: item.checksum.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_plan_views(items: &[DnsRecordPlan]) -> Vec<DnsRecordPlanView> {
    let mut views: Vec<DnsRecordPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_plan_view(item));
    }
    views
}

pub fn sum_dns_record_plan_view(view: &DnsRecordPlanView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_cost + (parsed_updated_at as f64) + (parsed_batch_size as f64))
}
