use super::*;

/// How screens and exports show a dns record cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCycleView {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub key_id: String,
    pub minimum_text: String,
    pub duration_minutes: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_dns_record_cycle_view(item: &DnsRecordCycle) -> DnsRecordCycleView {
    DnsRecordCycleView {
        code: item.code.clone(),
        title: item.title.clone(),
        summary: item.summary.clone(),
        key_id: item.key_id.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        duration_minutes: item.duration_minutes.to_string(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_cycle_views(items: &[DnsRecordCycle]) -> Vec<DnsRecordCycleView> {
    let mut views: Vec<DnsRecordCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_cycle_view(item));
    }
    views
}

pub fn sum_dns_record_cycle_view(view: &DnsRecordCycleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_minimum + (parsed_duration_minutes as f64) + parsed_budget)
}
