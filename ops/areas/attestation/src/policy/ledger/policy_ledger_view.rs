use super::*;

use acme_kit::text;

/// How screens and exports show a policy ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyLedgerView {
    pub code_text: String,
    pub label_text: String,
    pub tenant_id: String,
    pub region_text: String,
    pub weight_text: String,
    pub budget: String,
    pub quota_units: String,
    pub ttl_seconds: String,
    pub batch_size_text: String,
    pub segment: String,
    pub summary_text: String,
    pub status_label: String,
}

pub fn to_policy_ledger_view(item: &PolicyLedger) -> PolicyLedgerView {
    PolicyLedgerView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        tenant_id: item.tenant_id.clone(),
        region_text: text::truncate(&item.region, 40),
        weight_text: format!("{:.2}", item.weight),
        budget: format!("{:.2}", item.budget),
        quota_units: item.quota_units.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        batch_size_text: item.batch_size.to_string(),
        segment: item.segment.clone(),
        summary_text: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_ledger_views(items: &[PolicyLedger]) -> Vec<PolicyLedgerView> {
    let mut views: Vec<PolicyLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_ledger_view(item));
    }
    views
}

pub fn sum_policy_ledger_view(view: &PolicyLedgerView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_weight + parsed_budget + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_ledger("code-708", "label-824");
        let view = to_policy_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
