use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled egress policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyView {
    pub code: String,
    pub name_text: String,
    pub reviewer: String,
    pub cluster: String,
    pub checksum_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_scheduled_egress_policy_view(item: &ScheduledEgressPolicy) -> ScheduledEgressPolicyView {
    ScheduledEgressPolicyView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 120),
        reviewer: item.reviewer.clone(),
        cluster: item.cluster.clone(),
        checksum_text: text::truncate(&item.checksum, 64),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_egress_policy_views(items: &[ScheduledEgressPolicy]) -> Vec<ScheduledEgressPolicyView> {
    let mut views: Vec<ScheduledEgressPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_egress_policy_view(item));
    }
    views
}

pub fn sum_scheduled_egress_policy_view(view: &ScheduledEgressPolicyView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
