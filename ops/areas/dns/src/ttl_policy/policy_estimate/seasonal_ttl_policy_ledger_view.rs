use super::*;

/// How screens and exports show a seasonal ttl policy ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyLedgerView {
    pub code_text: String,
    pub label_text: String,
    pub discount: String,
    pub tenant_id: String,
    pub revision_text: String,
    pub host: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_seasonal_ttl_policy_ledger_view(item: &SeasonalTtlPolicyLedger) -> SeasonalTtlPolicyLedgerView {
    SeasonalTtlPolicyLedgerView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        tenant_id: item.tenant_id.clone(),
        revision_text: item.revision.to_string(),
        host: item.host.clone(),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_ttl_policy_ledger_views(items: &[SeasonalTtlPolicyLedger]) -> Vec<SeasonalTtlPolicyLedgerView> {
    let mut views: Vec<SeasonalTtlPolicyLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_ttl_policy_ledger_view(item));
    }
    views
}

pub fn sum_seasonal_ttl_policy_ledger_view(view: &SeasonalTtlPolicyLedgerView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_discount + (parsed_revision as f64))
}
