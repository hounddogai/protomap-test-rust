use super::*;

use acme_kit::text;

/// How screens and exports show a global dns record cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordCycleView {
    pub code: String,
    pub label: String,
    pub ceiling: String,
    pub updated_at: String,
    pub source: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_global_dns_record_cycle_view(item: &GlobalDnsRecordCycle) -> GlobalDnsRecordCycleView {
    GlobalDnsRecordCycleView {
        code: item.code.clone(),
        label: item.label.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        updated_at: item.updated_at.to_string(),
        source: text::truncate(&item.source, 120),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_dns_record_cycle_views(items: &[GlobalDnsRecordCycle]) -> Vec<GlobalDnsRecordCycleView> {
    let mut views: Vec<GlobalDnsRecordCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_dns_record_cycle_view(item));
    }
    views
}

pub fn sum_global_dns_record_cycle_view(view: &GlobalDnsRecordCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ceiling + (parsed_updated_at as f64) + parsed_amount)
}
