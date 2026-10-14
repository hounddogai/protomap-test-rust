use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive rule request.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleRequestView {
    pub code_text: String,
    pub name_text: String,
    pub quantity: String,
    pub issuer_text: String,
    pub amount: String,
    pub ttl_seconds: String,
    pub sequence: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_adaptive_rule_request_view(item: &AdaptiveRuleRequest) -> AdaptiveRuleRequestView {
    AdaptiveRuleRequestView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        quantity: item.quantity.to_string(),
        issuer_text: text::truncate(&item.issuer, 80),
        amount: format!("{:.2}", item.amount),
        ttl_seconds: item.ttl_seconds.to_string(),
        sequence: item.sequence.to_string(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_rule_request_views(items: &[AdaptiveRuleRequest]) -> Vec<AdaptiveRuleRequestView> {
    let mut views: Vec<AdaptiveRuleRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_rule_request_view(item));
    }
    views
}

pub fn sum_adaptive_rule_request_view(view: &AdaptiveRuleRequestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_amount + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_rule_request("code-680", "name-765");
        let view = to_adaptive_rule_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_rule_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
